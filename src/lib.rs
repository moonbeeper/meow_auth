use shadow_rs::shadow;

pub mod auth;
pub mod database;
pub mod global;
pub mod http;
pub mod job_queue;
pub mod logger;
pub mod mailer;
pub mod manager;
// pub mod oauth;
pub mod audit;
pub mod crypto;
pub mod settings;

shadow!(build);
