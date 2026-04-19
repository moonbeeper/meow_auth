use std::sync::LazyLock;

use data_encoding::BASE64_NOPAD;
use sqlx::PgPool;
use tera::Tera;

use crate::{
    job_queue::QueuedJob,
    mailer::{
        Email, MailerJob,
        error::{MailerErrors, MailerResult},
    },
};

#[derive(Debug, rust_embed::Embed)]
#[folder = "src/mailer/views/"]
#[include = "*.html"]
#[include = "*.txt"]
struct Templates;

pub static TERA: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    for filename in Templates::iter() {
        if let Some(file) = Templates::get(&filename) {
            let data =
                std::str::from_utf8(file.data.as_ref()).expect("valid utf-8 on html templates");

            tera.add_raw_template(&filename, data)
                .expect("failed adding templates")
        }
    }

    tera
});

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RawEmailTemplate {
    pub text_filename: Option<String>,
    pub html_filename: Option<String>,
    pub data: serde_json::Value,
}

pub struct RenderedTemplates {
    pub text: Option<String>,
    pub html: Option<String>,
}

impl RawEmailTemplate {
    pub fn render(&self) -> MailerResult<RenderedTemplates> {
        let context = tera::Context::from_value(self.data.clone())?;

        let txt = if let Some(v) = self.text_filename.as_ref() {
            Some(TERA.render(v, &context)?)
        } else {
            None
        };

        let html = if let Some(v) = self.html_filename.as_ref() {
            Some(TERA.render(v, &context)?)
        } else {
            None
        };

        Ok(RenderedTemplates { text: txt, html })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailTemplate<'a> {
    pub base: &'a str,
    pub data: serde_json::Value,
}

impl<'a> TryFrom<EmailTemplate<'a>> for RawEmailTemplate {
    type Error = MailerErrors;

    fn try_from(value: EmailTemplate) -> Result<Self, Self::Error> {
        let text_filename = format!("{}.txt", value.base);
        let html_filename = format!("{}.html", value.base);

        let has_text = Templates::get(&text_filename).is_some();
        let has_html = Templates::get(&html_filename).is_some();

        if !has_text && !has_html {
            tracing::error!("templates not found: {}", value.base);
            return Err(MailerErrors::TemplateNotFound(value.base.to_string()));
        }

        Ok(RawEmailTemplate {
            text_filename: has_text.then_some(text_filename),
            html_filename: has_html.then_some(html_filename),
            data: value.data,
        })
    }
}

pub trait MailerTemplate {
    fn mail_template(
        subject: String,
        to: String,
        template: EmailTemplate,
        db: &PgPool,
    ) -> impl Future<Output = MailerResult<()>> {
        async move {
            let raw = RawEmailTemplate::try_from(template)?;
            let data = serde_json::to_vec(&raw)?;
            let data = BASE64_NOPAD.encode(&data);

            let email = Email::builder()
                .template(Some(data))
                .subject(subject)
                .to(to)
                .build();

            MailerJob::dispatch(db, email).await.map_err(|e| {
                tracing::error!("failed dispatching job: {e}");
                MailerErrors::JobQueue(e)
            })?;

            Ok(())
        }
    }
}
