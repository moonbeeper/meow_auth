use std::sync::Arc;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::MultiPart,
    transport::smtp::authentication::Credentials,
};

use crate::{
    global::GlobalState,
    job_queue::QueuedJob,
    settings::{MailerSettings, Settings},
};

#[derive(Debug)]
pub struct Mailer {
    transport: Arc<AsyncSmtpTransport<Tokio1Executor>>,
    settings: MailerSettings,
}

impl Mailer {
    pub async fn new(settings: &Settings) -> anyhow::Result<Self> {
        let mut transport_builder = match settings.mailer.smtp_secure {
            true => AsyncSmtpTransport::<Tokio1Executor>::relay(&settings.mailer.smtp_host)
                .map_err(|e| {
                    tracing::error!("something went wrong while setting up the mailer: {}", e);
                    e
                })?,
            false => {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&settings.mailer.smtp_host)
            }
        };

        let credentials = Credentials::new(
            settings.mailer.smtp_username.clone(),
            settings.mailer.smtp_password.clone(),
        );

        transport_builder = transport_builder.credentials(credentials);
        transport_builder = transport_builder.port(settings.mailer.smtp_port);

        let transport = transport_builder.build();
        if settings.mailer.test_connection {
            transport.test_connection().await.map_err(|e| {
                tracing::error!("failed testing connection to mail srv: {}", e);
                e
            })?;
        }

        Ok(Self {
            transport: Arc::new(transport),
            settings: settings.mailer.clone(),
        })
    }

    pub async fn mail(&self, email: &Email) -> anyhow::Result<()> {
        let m = Message::builder()
            .from(format!("System <{}>", self.settings.from_email).parse()?)
            .to(email.to.parse()?)
            .subject(email.subject.clone());

        let msg = if let Some(html) = email.html.as_ref() {
            m.multipart(MultiPart::alternative_plain_html(
                email.text.clone(),
                html.clone(),
            ))?
        } else {
            m.body(email.text.clone())?
        };

        self.transport.send(msg).await?;
        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, typed_builder::TypedBuilder)]
pub struct Email {
    pub to: String,
    #[builder(default = "no subject".to_string())]
    pub subject: String,
    pub text: String,
    #[builder(default)]
    pub html: Option<String>,
}

pub struct MailerJob;

impl QueuedJob for MailerJob {
    type Input = Email;

    async fn run(&self, global: Arc<GlobalState>, input: Self::Input) -> anyhow::Result<()> {
        match global.mailer.mail(&input).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                tracing::error!("something went wrong while trying to send an email: {e}");
                Err(e)?
            }
        }
    }
}
