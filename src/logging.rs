use crate::settings::{Logging, LoggingFormat};
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init(settings: &Logging) {
    if !settings.enabled {
        return;
    }

    let mut layers = Vec::new();
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::try_new(format!("{:?}", settings.level))
            .expect("You provided an invalid logging level?!?!?")
    });
    let fmt_layer = match settings.format {
        LoggingFormat::Full => fmt::layer().boxed(),
        LoggingFormat::Compact => fmt::layer().compact().boxed(),
        LoggingFormat::Pretty => fmt::layer().pretty().boxed(),
        LoggingFormat::Json => fmt::layer().json().boxed(),
    };
    layers.push(fmt_layer);

    if settings.file.enabled {
        let log_layer = tracing_appender::rolling::Builder::default()
            .max_log_files(settings.file.max_count)
            .rotation(Rotation::DAILY)
            .build(&settings.file.path)
            .expect("Couldn't build logging layer");
        let (non_blocking, guard) = tracing_appender::non_blocking(log_layer);
        let log_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);
        let log_layer = match settings.format {
            LoggingFormat::Full => log_layer.boxed(),
            LoggingFormat::Compact => log_layer.compact().boxed(),
            LoggingFormat::Pretty => log_layer.pretty().boxed(),
            LoggingFormat::Json => log_layer.json().boxed(),
        };

        let _ = LOG_GUARD.get_or_init(|| guard);
        layers.push(log_layer);
    }

    tracing_subscriber::registry()
        .with(layers)
        .with(filter_layer)
        .init();

    tracing::info!("Hi there :)");
}
