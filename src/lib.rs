use shadow_rs::shadow;

pub mod audit;
pub mod auth;
pub mod crypto;
pub mod database;
pub mod global;
pub mod http;
pub mod job_queue;
pub mod logger;
pub mod mailer;
pub mod manager;
pub mod oauth;
pub mod ratelimiter;
pub mod settings;

shadow!(build);

doc_node_ignore!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    bool,
    char,
    String,
    &str,
    settings::LoggingLevel,
    url::Url,
    std::net::SocketAddr,
    crypto::SecretKey,
    database::id::UlidId,
    std::borrow::Cow<'static, str>
);
