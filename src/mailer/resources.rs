use std::sync::LazyLock;

use data_encoding::BASE64_NOPAD;
use minify_html::minify;
use rand::seq::IndexedRandom;
use serde_json::json;
use sqlx::PgPool;
use tera::Tera;

use crate::{
    job_queue::QueuedJob,
    mailer::{
        Email, MailSettings, MailerJob,
        error::{MailerErrors, MailerResult},
    },
};

#[derive(Debug, rust_embed::Embed)]
#[folder = "src/mailer/views/"]
#[include = "**/*.html"]
#[include = "**/*.txt"]
struct Templates;

pub static TERA: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    let mut templates = Vec::new();

    for filename in Templates::iter() {
        if let Some(file) = Templates::get(&filename) {
            let data = std::str::from_utf8(file.data.as_ref())
                .expect("valid utf-8 on html templates")
                .to_owned();

            templates.push((filename.into_owned(), data));
        }
    }

    tera.add_raw_templates(templates)
        .expect("failed adding templates");
    tera.autoescape_on(vec![".html", ".txt"]);
    // tera.register_function("get_greeting", get_greeting);

    tera
});

const HTML_MINIFY_CFG: minify_html::Cfg = minify_html::Cfg {
    minify_css: true,
    keep_html_and_head_opening_tags: true,
    allow_noncompliant_unquoted_attribute_values: false,
    allow_optimal_entities: false,
    allow_removing_spaces_between_attributes: false,
    keep_closing_tags: false,
    keep_comments: true, // mso stuff? goddamn i hate the stupid email making stuff ive chosen
    keep_input_type_text_attr: false,
    keep_ssi_comments: false,
    minify_doctype: false,
    minify_js: false,
    preserve_brace_template_syntax: false,
    preserve_chevron_percent_template_syntax: false,
    remove_bangs: false,
    remove_processing_instructions: false,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RawEmailTemplate {
    pub text_filename: Option<String>,
    pub html_filename: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct RenderedTemplates {
    pub text: Option<String>,
    pub html: Option<String>,
}

impl RawEmailTemplate {
    pub fn render(&self, mail_settings: &MailSettings) -> MailerResult<RenderedTemplates> {
        let mut context = tera::Context::from_value(self.data.clone())?;
        context.insert(
            "globals",
            &json!({
                "origin": mail_settings.http.origin.to_string(),
                "greeting": get_greeting(),
            }),
        );

        let txt = if let Some(v) = self.text_filename.as_ref() {
            let rendered = TERA.render(v, &context)?;
            Some(rendered)
        } else {
            None
        };

        let html = if let Some(v) = self.html_filename.as_ref() {
            let rendered = TERA.render(v, &context)?;
            let minified = minify(rendered.as_bytes(), &HTML_MINIFY_CFG);
            match std::str::from_utf8(&minified) {
                Ok(v) => Some(v.to_owned()),
                Err(e) => {
                    tracing::error!("failed to convert minified text template to UTF-8: {e}");
                    return Err(MailerErrors::Utf8Error(e));
                }
            }
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

const EMAIL_GREETINGS: [&str; 8] = [
    "Hey",
    "Hi",
    "Hello",
    "Howdy",
    "Ahoy",
    "Good to see you",
    "Meow",
    "Hola",
];

// pub fn get_greeting(_: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
//     let mut rng = rand::rng();
//     let greeting = EMAIL_GREETINGS.choose(&mut rng).copied().unwrap();
//     Ok(tera::to_value(greeting.to_string()).unwrap())
// }
//
pub fn get_greeting() -> String {
    let mut rng = rand::rng();
    let greeting = EMAIL_GREETINGS.choose(&mut rng).copied().unwrap();
    greeting.to_string()
}
