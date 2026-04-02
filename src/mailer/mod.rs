use std::sync::Arc;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    transport::smtp::authentication::Credentials,
};

use crate::settings::Settings;

// TODO: should have a bg worker for sending emails. This would let routes return faster by just queuing emails to be sent by these workers.

#[derive(Debug)]
pub struct Mailer {
    transport: Arc<AsyncSmtpTransport<Tokio1Executor>>,
    from_email: String,
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
            from_email: settings.mailer.from_email.clone(),
        })
    }

    pub async fn mail(&self, to: String, text: String) -> anyhow::Result<()> {
        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to.parse()?)
            .body(text)?;

        self.transport.send(email).await?;
        Ok(())
    }
}
