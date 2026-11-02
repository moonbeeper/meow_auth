use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::settings::{Logging, LoggingFormat, OtlpSettings};

pub fn init(settings: &Logging) -> Option<SdkTracerProvider> {
    if !settings.enabled {
        return None;
    }

    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(settings.level.to_string());

    let trace_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(settings.otlp.level.to_string());

    let format = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true);
    let format = match settings.format {
        LoggingFormat::Json => format.json().boxed(),
        LoggingFormat::Human => format.pretty().boxed(),
    };
    let otlp = otlp_provider(&settings.otlp);
    let registry = tracing_subscriber::registry().with(format.with_filter(filter_layer));

    match &otlp {
        Some(provider) => {
            let tracer = provider.tracer("meow_auth2");
            let tracer_layer = tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(trace_filter);
            registry.with(tracer_layer).init();
        }
        None => registry.init(),
    }

    otlp
}

fn otlp_provider(settings: &OtlpSettings) -> Option<SdkTracerProvider> {
    if !settings.enabled || settings.endpoint.is_none() {
        return None;
    }

    let mut builder = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary);

    if let Some(endpoint) = &settings.endpoint {
        builder = builder.with_endpoint(endpoint);
    }

    let exporter = match builder.build() {
        Ok(exporter) => exporter,
        Err(e) => {
            tracing::error!(error = %e, "failed to build OTLP exporter");
            return None;
        }
    };

    Some(
        SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(Resource::builder().with_service_name("meow_auth2").build())
            .build(),
    )
}
