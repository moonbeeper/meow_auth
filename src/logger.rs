use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::settings::{Logging, LoggingFormat};

pub fn init(settings: &Logging) {
    if !settings.enabled {
        return;
    }

    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(settings.level.to_string());

    let format = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true);
    let format = match settings.format {
        LoggingFormat::Json => format.json().boxed(),
        LoggingFormat::Human => format.pretty().boxed(),
    };

    tracing_subscriber::registry()
        .with(format.with_filter(filter_layer))
        .init();
}
