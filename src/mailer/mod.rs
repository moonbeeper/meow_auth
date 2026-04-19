pub mod error;
pub mod resources;

use std::sync::Arc;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::MultiPart,
    transport::smtp::authentication::Credentials,
};

use crate::{
    global::GlobalState,
    job_queue::QueuedJob,
    mailer::{error::MailerResult, resources::RawEmailTemplate},
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

    pub async fn mail(&self, email: &Email) -> MailerResult<()> {
        let m = Message::builder()
            .from(format!("System <{}>", self.settings.from_email).parse()?)
            .to(email.to.parse()?)
            .subject(email.subject.clone());

        let mut text = email.text.clone();
        let mut html = email.html.clone();

        if let Some(templates) = email.template.as_ref() {
            let rendered = templates.render()?;
            if text.is_none() && rendered.text.is_none() {
                text = Some("no provided message".to_string());
            } else if text.is_none() {
                text = rendered.text;
            }

            if html.is_none() && rendered.html.is_none() {
                html = None;
            } else if html.is_none() {
                html = rendered.html;
            }
        }

        let text = text.unwrap_or_else(|| "no provided message".to_string());

        let msg = if let Some(html) = html.as_ref() {
            m.multipart(MultiPart::alternative_plain_html(text, html.clone()))?
        } else {
            m.body(text)?
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
    #[builder(default = Some("no provided message".to_string()))]
    pub text: Option<String>,
    #[builder(default = None)]
    pub html: Option<String>,
    #[builder(default = None)]
    pub template: Option<RawEmailTemplate>,
}

pub struct MailerJob;

impl QueuedJob for MailerJob {
    type Input = Email;

    async fn run(&self, global: Arc<GlobalState>, input: Self::Input) -> anyhow::Result<()> {
        match global.mailer.mail(&input).await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::error!("something went wrong while trying to send an email: {e}");
                Err(e)?
            }
        }
    }
}
