use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::http::{HeaderMap, HeaderName, header::RETRY_AFTER};
use dashmap::DashMap;

pub const RATELIMIT_LIMIT: HeaderName = HeaderName::from_static("x-ratelimit-limit");
pub const RATELIMIT_USED: HeaderName = HeaderName::from_static("x-ratelimit-used");
pub const RATELIMIT_REMAINING: HeaderName = HeaderName::from_static("x-ratelimit-remaining");
pub const RATELIMIT_RESET: HeaderName = HeaderName::from_static("x-ratelimit-reset");

type StateMap = DashMap<IpAddr, TicketState>;

#[derive(Debug)]
pub struct TicketState {
    tickets: u64,
    last_refill_at: Instant,
}

#[derive(Debug)]
pub struct Ratelimiter {
    pub max_tickets: u64,
    pub refill_after: Duration,
    state: Arc<StateMap>,
    _guard: tokio_util::sync::DropGuard,
}

pub struct RatelimiterResponse {
    /// Whether the request is allowed to pass or not.
    ///
    /// Not allowed when the ip has used all of its tickets and is waiting for refill time
    pub allowed: bool,
    /// The max amount of tickets that can be used
    limit: u64,
    /// The amount of tickets that have been usde
    used: u64,
    /// The amount of tickets that are remaining (limit - used)
    remaining: u64,
    /// The amount of seconds until the tickets are refilled one by one
    reset: u64,
}

impl Ratelimiter {
    pub fn new(max_tickets: u64, refill_after: Duration) -> Self {
        let dashmap = Arc::new(DashMap::new());
        let guard = tokio_util::sync::CancellationToken::new();

        tokio::spawn(garbage(dashmap.clone(), refill_after, guard.child_token()));
        Self {
            max_tickets,
            refill_after,
            state: dashmap,
            _guard: guard.drop_guard(),
        }
    }

    pub fn adquire(&self, ip: IpAddr) -> RatelimiterResponse {
        let now = Instant::now();
        let mut entry = self.state.entry(ip).or_insert(TicketState {
            tickets: self.max_tickets,
            last_refill_at: now,
        });

        let since_last_refill = now.duration_since(entry.last_refill_at);
        let refill_count =
            (since_last_refill.as_secs_f64() / self.refill_after.as_secs_f64()).floor() as u64;

        let refill_count = refill_count.min(self.max_tickets); // DUMB BIRD. ITS BACKWARDS!!!! MIN = MAX ; MAX = MIN
        if refill_count > 0 {
            entry.tickets = entry
                .tickets
                .saturating_add(refill_count)
                // god DADMMIT FUCKING SHIT bird I AM GODDAMIT AGH. if an user has more than the max tickets.. we still add more.
                // AND if we still add more, ANDDD this is STILL a u64 NOT to be confused with a I64. We underflow it lol in the sub step below.
                // Practically, "left number small and right number small equals negative numbers which equals to poo poo in u64"
                .min(self.max_tickets);
            entry.last_refill_at = now;
        }

        if entry.tickets == 0 {
            return RatelimiterResponse {
                allowed: false,
                limit: self.max_tickets,
                used: self.max_tickets,
                remaining: 0,
                reset: self.refill_after.as_secs(),
            };
        }

        entry.tickets = entry.tickets.saturating_sub(1);
        let used_tickets = self.max_tickets - entry.tickets;

        RatelimiterResponse {
            allowed: true,
            limit: self.max_tickets,
            used: used_tickets,
            remaining: entry.tickets,
            reset: self.refill_after.as_secs(),
        }
    }
}

// crap https://news.ycombinator.com/item?id=46618105
impl RatelimiterResponse {
    pub fn header_map(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(RATELIMIT_LIMIT, self.limit.into());
        headers.insert(RATELIMIT_USED, self.used.into());
        headers.insert(RATELIMIT_REMAINING, self.remaining.into());
        headers.insert(RATELIMIT_RESET, self.reset.into());

        if !self.allowed {
            headers.insert(RETRY_AFTER, self.reset.into());
        }
        headers
    }
}

// garbage collection for the ip state map. it just removes old entries lol
async fn garbage(
    dashmap: Arc<StateMap>,
    refill_after: Duration,
    guard: tokio_util::sync::CancellationToken,
) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60 * 5));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                dashmap.retain(|_, v| v.last_refill_at.elapsed() < refill_after * 3);
            }

            _ = guard.cancelled() => break,
        }
    }
}
