use crate::job_queue::JobQueueErrors;

pub type MailerResult<T> = Result<T, MailerErrors>;

#[derive(Debug, thiserror::Error)]
pub enum MailerErrors {
    #[error("failed to render template: {0}")]
    TemplateRender(#[from] tera::Error),
    #[error("template not found: {0}")]
    TemplateNotFound(String),
    #[error("failed to parse email address: {0}")]
    EmailParse(#[from] lettre::address::AddressError),
    #[error("failed to build email: {0}")]
    EmailBuilder(#[from] lettre::error::Error),
    #[error("failed to send email: {0}")]
    EmailSend(#[from] lettre::transport::smtp::Error),
    #[error("failed to dispatch job: {0}")]
    JobQueue(#[from] JobQueueErrors),
    #[error("failed to decode/decode template data: {0}")]
    TemplateData(#[from] serde_json::Error),
    #[error("failed to decode base64 template data: {0}")]
    Base64TemplateData(#[from] data_encoding::DecodeError),
}
