use std::str::FromStr;

use mediator::Module;
use serde::Deserialize;
use tracing::{metadata::LevelFilter, trace, Level};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, Layer};

#[cfg(feature = "export-tracing")]
pub use tracing;
#[cfg(feature = "export-tracing")]
pub use tracing_subscriber;

#[derive(Deserialize)]
pub enum TracingConfig {
    Log { level: String },
    Otel,
}

impl Default for TracingConfig {
    fn default() -> Self {
        TracingConfig::Log {
            level: "debug".to_string(),
        }
    }
}

#[derive(Default)]
pub struct TracingModule {
    config: TracingConfig,
}

pub struct UninitializedTracingModule;

impl Module for UninitializedTracingModule {
    type Config = Option<TracingConfig>;

    type Out = TracingModule;

    fn initialize(self, config: Self::Config) -> Self::Out {
        TracingModule {
            config: config.unwrap_or_default(),
        }
    }
}

impl TracingModule {
    pub fn new() -> UninitializedTracingModule {
        UninitializedTracingModule
    }

    pub fn from_level<S: ToString>(level: S) -> Self {
        Self {
            config: TracingConfig::Log {
                level: level.to_string(),
            },
        }
    }

    pub fn init(self) {
        match self.config {
            TracingConfig::Log { level } => {
                tracing_subscriber::registry().with(
                    fmt::layer()
                        .with_ansi(true)
                        .compact()
                        .with_filter(LevelFilter::from(Level::from_str(level.as_str()).unwrap())),
                );
            }
            TracingConfig::Otel => {
                let otlp_tracer = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
                    .install_simple()
                    .expect("Unable to initialize tracer.");
                tracing_subscriber::registry().with(OpenTelemetryLayer::new(otlp_tracer));
            }
            _ => todo!(),
        }
        trace!("Tracing initialized");
    }
}
