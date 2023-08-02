use std::{collections::HashMap, str::FromStr};

use mediator::Module;
use serde::Deserialize;
use strum::AsRefStr;
use tracing::{metadata::LevelFilter, trace, Level};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[cfg(feature = "export-tracing")]
pub use tracing;
#[cfg(feature = "export-tracing")]
pub use tracing_subscriber;

pub use tracing_subscriber::filter::Targets;

#[derive(Deserialize, Default)]
pub struct RuntimeTracingConfigOverride {
    default: Option<TracingLevel>,
    targets: Option<TracingFilterTargets>,
    layer: Option<TracingLayer>,
}

#[derive(Deserialize)]
pub enum TracingLayer {
    Log,
    Otel,
}

#[derive(Deserialize, AsRefStr)]
pub enum TracingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl From<TracingLevel> for LevelFilter {
    fn from(level: TracingLevel) -> Self {
        LevelFilter::from_str(level.as_ref()).unwrap()
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TracingFilterTargets {
    Single(String),
    Map(HashMap<String, TracingLevel>),
}

#[derive(Default)]
pub struct TracingModule {
    config: TracingConfig,
}

pub struct TracingConfig {
    pub base_targets: Option<Targets>,
    pub layer: TracingLayer,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            base_targets: Some(Targets::default().with_default(Level::INFO)),
            layer: TracingLayer::Log,
        }
    }
}

impl TracingConfig {
    fn with_overrides(self, runtime_config: RuntimeTracingConfigOverride) -> Self {
        let base_targets = self.base_targets.map(|mut base| {
            if let Some(targets) = runtime_config.targets {
                let runtime_targets = match targets {
                    TracingFilterTargets::Single(targets_string) => {
                        targets_string.parse::<Targets>().unwrap()
                    }
                    TracingFilterTargets::Map(targets) => targets
                        .into_iter()
                        .fold(Targets::default(), |targets, (target, level)| {
                            targets.with_target(target, level)
                        }),
                };

                base = base.with_targets(runtime_targets);
            }

            if let Some(runtime_default) = runtime_config.default {
                base = base.with_default(runtime_default);
            }

            base
        });

        Self {
            base_targets,
            layer: runtime_config.layer.unwrap_or(self.layer),
        }
    }
}

pub struct UninitializedTracingModule {
    initial_config: TracingConfig,
}

impl Module for UninitializedTracingModule {
    type Config = Option<RuntimeTracingConfigOverride>;

    type Out = TracingModule;

    fn initialize(self, overrides: Self::Config) -> Self::Out {
        let mut config = self.initial_config;
        if let Some(overrides) = overrides {
            config = config.with_overrides(overrides);
        }

        TracingModule { config }
    }
}

impl TracingModule {
    pub fn new(initial_config: Option<TracingConfig>) -> UninitializedTracingModule {
        UninitializedTracingModule {
            initial_config: initial_config.unwrap_or_default(),
        }
    }

    pub fn init(self) {
        let trace_targets: Targets = self.config.base_targets.unwrap_or_default();

        match self.config.layer {
            TracingLayer::Log => {
                tracing_subscriber::registry()
                    .with(
                        fmt::layer()
                            .with_ansi(true)
                            .compact()
                            .with_filter(trace_targets),
                    )
                    .init();
            }
            TracingLayer::Otel => {
                let otlp_tracer = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
                    .install_simple()
                    .expect("Unable to initialize otlp tracer.");
                tracing_subscriber::registry()
                    .with(OpenTelemetryLayer::new(otlp_tracer).with_filter(trace_targets))
                    .init();
            }
            _ => todo!(),
        }
        trace!("Tracing initialized");
    }
}
