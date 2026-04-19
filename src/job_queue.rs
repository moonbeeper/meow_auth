use std::{
    collections::HashMap,
    panic::AssertUnwindSafe,
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use chrono::Utc;
use futures_util::{FutureExt, future::BoxFuture};
use sqlx::{PgPool, postgres::PgListener};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

use crate::{database::id::UlidId, global::GlobalState, manager::WatcherChild};

type JobQueueResult<T> = std::result::Result<T, JobQueueErrors>;

#[derive(Debug, thiserror::Error)]
pub enum JobQueueErrors {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("requested job handler not registered: {0}")]
    HandlerNotRegistered(String),
    #[error("worker panicked: {0}")]
    WorkerPanic(String),
    #[error("job input serialization failed: {0}")]
    Serialize(#[from] postcard::Error),
    #[error("job input deserialization failed for handler {handler}: {source}")]
    Deserialize {
        handler: String,
        #[source]
        source: postcard::Error,
    },
    #[error("worker error: {0}")]
    WorkerError(#[from] anyhow::Error),
    #[error("failed to dispatch job: {0}")]
    Dispatch(Box<JobQueueErrors>),
}

pub trait QueuedJob: Send + Sync + 'static {
    type Input: serde::Serialize + serde::de::DeserializeOwned + Send + 'static;

    fn run(
        &self,
        global: Arc<GlobalState>,
        input: Self::Input,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
    fn expires_in() -> chrono::Duration {
        chrono::Duration::days(1)
    }
    fn dispatch(pool: &PgPool, input: Self::Input) -> impl Future<Output = JobQueueResult<()>> {
        async move {
            let input = postcard::to_allocvec(&input)
                .map_err(JobQueueErrors::from)
                .map_err(|e| JobQueueErrors::Dispatch(Box::new(e)))?;

            sqlx::query!(
                "insert into
                    queued_jobs (id, handler, state, input, expires_at)
                values
                    ($1, $2, $3, $4, $5)",
                QueuedJobId::new() as QueuedJobId,
                Self::name().to_string(),
                QueuedJobState::Pending.as_str(),
                input,
                Utc::now() + Self::expires_in()
            )
            .execute(pool)
            .await
            .map_err(JobQueueErrors::from)
            .map_err(|e| JobQueueErrors::Dispatch(Box::new(e)))?;

            Ok(())
        }
    }
}

type JobHandler = Box<dyn Fn(Vec<u8>) -> BoxFuture<'static, JobQueueResult<()>> + Send + Sync>;

pub struct QueueRegistry {
    job_handlers: HashMap<&'static str, JobHandler>,
    global: Arc<GlobalState>,
    concurrency: usize,
    batch_size: usize,
    heartbeat_interval: Duration,
}

impl QueueRegistry {
    pub fn new(global: Arc<GlobalState>) -> Self {
        Self {
            job_handlers: HashMap::new(),
            global,
            concurrency: 50,
            batch_size: 1000,
            heartbeat_interval: Duration::from_secs(60),
        }
    }

    pub fn register<J: QueuedJob>(mut self, job: J) -> Self {
        let job = Arc::new(job);
        let global = self.global.clone();
        let handler = Box::new(move |value: Vec<u8>| {
            let job = job.clone();
            let global = global.clone();
            Box::pin(async move {
                let input: J::Input =
                    postcard::from_bytes(&value).map_err(|e| JobQueueErrors::Deserialize {
                        handler: J::name().to_string(),
                        source: e,
                    })?;

                match AssertUnwindSafe(job.run(global, input))
                    .catch_unwind()
                    .await
                {
                    Ok(res) => res.map_err(|e| JobQueueErrors::WorkerError(e)),
                    Err(panic) => {
                        let panic_msg = panic
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .or_else(|| panic.downcast_ref::<&str>().copied())
                            .unwrap_or("Unknown panic occurred")
                            .to_string();

                        tracing::error!(err = panic_msg, "worker panicked");
                        Err(JobQueueErrors::WorkerPanic(panic_msg))
                    }
                }
            }) as BoxFuture<'static, JobQueueResult<()>>
        });

        self.job_handlers.insert(J::name(), handler);
        self
    }

    pub fn set_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    pub fn set_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub async fn run(self, shutdown: WatcherChild) -> JobQueueResult<()> {
        Self::run_each(self, Duration::from_mins(2), shutdown).await
    }

    pub async fn run_each(self, tick: Duration, shutdown: WatcherChild) -> JobQueueResult<()> {
        tracing::info!(
            "creating queue registry with a batch size of {} and a concurrency of {}",
            self.batch_size,
            self.concurrency
        );
        tracing::info!(
            "queue registry has these handlers registered: {:?}",
            self.job_handlers.keys().collect::<Vec<_>>()
        );

        let mut tick = tokio::time::interval(tick);
        let mut listener = PgListener::connect_with(&self.global.database).await?;
        listener.listen("added_queued_job").await?;

        let worker = Arc::new(Worker::new(
            self.concurrency,
            self.batch_size,
            self.heartbeat_interval,
            self.job_handlers,
            shutdown.token().child_token(),
            self.global.clone(),
        ));

        worker.claim_available().await?;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,

                _ = tick.tick() => {
                    worker.claim_available().await?;
                }

                msg = listener.recv() => {
                    match msg {
                        Ok(_) => {
                            worker.claim_available().await?;
                        }
                        Err(e) => {
                            tracing::error!("something went wrong with our pglistener: {e}")
                        }
                    }
                }
            }
        }

        tracing::info!("waiting for queue workers to finish");
        worker.wait_for_idle().await;
        tracing::info!("goodnight little workers");
        Ok(())
    }
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum QueuedJobState {
    #[default]
    Pending,
    InProgress,
    Success,
    Failed,
}

impl QueuedJobState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "inprogress",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }
}

type QueuedJobId = UlidId;

struct QueuedJobData {
    id: UlidId,
    handler: String,
    input: Vec<u8>,
}

struct Worker {
    job_handlers: HashMap<&'static str, JobHandler>,
    global: Arc<GlobalState>,
    semaphore: Arc<Semaphore>,
    batch_size: usize,
    token: CancellationToken,
    heartbeat_interval: Duration,
    active_workers: Arc<AtomicUsize>,
}

impl Worker {
    pub fn new(
        concurrency: usize,
        batch_size: usize,
        heartbeat_interval: Duration,
        job_handlers: HashMap<&'static str, JobHandler>,
        token: CancellationToken,
        global: Arc<GlobalState>,
    ) -> Self {
        Self {
            job_handlers,
            global,
            semaphore: Arc::new(Semaphore::new(concurrency)),
            batch_size,
            token,
            heartbeat_interval,
            active_workers: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn claim_available(self: &Arc<Self>) -> JobQueueResult<()> {
        loop {
            if self.token.is_cancelled() {
                break;
            }

            let jobs = self.claim_next_batch().await?;
            if jobs.is_empty() {
                break;
            }

            for task in jobs {
                let Ok(ticket) = self.semaphore.clone().acquire_owned().await else {
                    return Ok(()); // the error is only returned when the semaphore is closed.
                };

                let this = self.clone();
                tokio::spawn(async move {
                    let run = this.run(task, ticket).await;
                    if let Err(e) = run {
                        tracing::error!("something went wrong while running the worker: {e}");
                    }
                });
            }
        }
        Ok(())
    }

    pub async fn claim_next_batch(&self) -> JobQueueResult<Vec<QueuedJobData>> {
        let mut tx = self.global.database.begin().await?;
        let query = sqlx::query!(
            "with pending as (
              select id, created_at
              from queued_jobs
              where
                state = $1
                and success_at is null
                and retry_count < retry_max_count
                AND expires_at > now()
              order by created_at
              for update skip locked
              limit $4
            ),
            stale_inprogress as (
              select id, updated_at
              from queued_jobs
              where
                state = $2
                and updated_at < now() - make_interval(secs => $3)
                and success_at is null
                and retry_count < retry_max_count
                and expires_at > now()
              order by updated_at
              for update skip locked
              limit GREATEST(0, 1000 - (select count(*) from pending))
            ),
            picked as (
              select id FROM pending
              UNION ALL
              select id FROM stale_inprogress
            )
            update queued_jobs job
            set state = $2, updated_at = now()
            from picked
            where job.id = picked.id
            returning job.id, job.handler, job.input",
            QueuedJobState::Pending.as_str(),
            QueuedJobState::InProgress.as_str(),
            self.heartbeat_interval.as_secs_f64(),
            self.batch_size as i64
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        let result = query
            .into_iter()
            .map(|v| QueuedJobData {
                id: v.id.into(),
                handler: v.handler,
                input: v.input,
            })
            .collect();
        Ok(result)
    }

    pub async fn run(
        &self,
        job: QueuedJobData,
        _ticket: OwnedSemaphorePermit,
    ) -> JobQueueResult<()> {
        let _guard = _ticket;
        let Some(handler) = self.job_handlers.get(job.handler.as_str()) else {
            self.release_fail(
                job.id,
                true,
                Some("the requested handler is not registered".to_string()),
            )
            .await?;
            return Ok(());
        };

        let job_id = job.id;
        let pool = self.global.database.clone();
        let heartbeat_interval = self.heartbeat_interval;
        let heartbeat = tokio::spawn(async move {
            let mut tick = tokio::time::interval(heartbeat_interval);
            tick.tick().await;
            loop {
                tick.tick().await;
                Self::heartbeat(job_id, &pool).await.unwrap_or_else(|_| {
                    tracing::error!(
                        job_id = job_id.to_string(),
                        "failed to save heartbeat for job"
                    )
                });
            }
        });

        self.active_workers
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        match handler(job.input).await {
            Ok(()) => self.release_success(job.id).await?,
            Err(e) => {
                self.release_fail(job.id, false, Some(e.to_string()))
                    .await?
            }
        };

        heartbeat.abort();
        self.active_workers
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    pub async fn release_success(&self, job_id: QueuedJobId) -> JobQueueResult<()> {
        sqlx::query!(
            "update queued_jobs set state = $2, success_at = now(), updated_at = now() where id = $1",
            job_id as QueuedJobId,
            QueuedJobState::Success.as_str()
        )
        .execute(&self.global.database)
        .await?;
        Ok(())
    }

    pub async fn release_fail(
        &self,
        job_id: QueuedJobId,
        no_retry: bool,
        message: Option<String>,
    ) -> JobQueueResult<()> {
        sqlx::query!(
            "update queued_jobs
                set retry_count = retry_count+1,
                    state = case
                        when $4 then $2
                        when retry_count+1 >= retry_max_count then $2 else $3 end,
                    updated_at = now(),
                    error_message = $5
            where id = $1
            ",
            job_id as QueuedJobId,
            QueuedJobState::Failed.as_str(),
            QueuedJobState::Pending.as_str(),
            no_retry,
            message.as_ref()
        )
        .execute(&self.global.database)
        .await?;
        Ok(())
    }

    pub async fn heartbeat(job_id: QueuedJobId, pool: &PgPool) -> JobQueueResult<()> {
        sqlx::query!(
            "update queued_jobs set updated_at = now() where id = $1",
            job_id as QueuedJobId,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn wait_for_idle(&self) {
        self.semaphore.close();
        loop {
            if self
                .active_workers
                .load(std::sync::atomic::Ordering::SeqCst)
                == 0
            {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
