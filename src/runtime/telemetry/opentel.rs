//! OpenTelemetry initialisation related logic.
use anyhow::Result;
use opentelemetry::propagation::TextMapCompositePropagator;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation;
use opentelemetry_sdk::trace::Sampler as SdkSampler;
use opentelemetry_sdk::trace::BatchSpanProcessor;
use serde::Deserialize;
use serde::Serialize;

/// Configuration options for process telemetry data using OpenTelemetry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OTelConfig {
    /// Enable export of telemetry data.
    #[serde(default = "OTelConfig::default_enabled")]
    pub enabled: bool,

    /// GRPC endpoint to export OpenTelemetry data to.
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Configure sampling of traces.
    #[serde(default)]
    pub sampling: Sampler,

    /// Timeout in seconds when communicating with the OpenTelemetry agent.
    #[serde(default)]
    pub timeout_sec: Option<u64>,
}

impl Default for OTelConfig {
    fn default() -> Self {
        OTelConfig {
            enabled: OTelConfig::default_enabled(),
            endpoint: None,
            sampling: Sampler::default(),
            timeout_sec: None,
        }
    }
}

impl OTelConfig {
    fn default_enabled() -> bool {
        false
    }
}

/// Programmatic options for the OpenTelemetry framework.
#[derive(Default)]
pub struct OTelOptions {
    /// Configuration for the batch exporter.
    pub batch_config: Option<opentelemetry_sdk::trace::BatchConfig>,

    /// Attributes representing the process that produces telemetry data.
    pub resource: opentelemetry_sdk::Resource,
}

/// Trace sampling configuration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Sampler {
    /// Follow the sampling decision of the parent span, if any exists.
    #[serde(default)]
    pub follow_parent: bool,

    /// The sampling rule for traces without a parent span.
    #[serde(default)]
    pub mode: SamplerMode,
}

impl From<Sampler> for SdkSampler {
    fn from(value: Sampler) -> Self {
        let mode: SdkSampler = value.mode.into();
        if value.follow_parent {
            SdkSampler::ParentBased(Box::new(mode))
        } else {
            mode
        }
    }
}

/// The trace sampling mode for traces without a parent span.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum SamplerMode {
    /// Always sample new traces.
    #[default]
    #[serde(alias = "ALWAYS", alias = "always")]
    Always,

    /// Never sample new traces.
    #[serde(alias = "NEVER", alias = "NEVER")]
    Never,

    /// Sample a portion of traces based on the configured ratio.
    #[serde(alias = "RATIO", alias = "RATIO")]
    Ratio(f64),
}

impl From<SamplerMode> for SdkSampler {
    fn from(value: SamplerMode) -> Self {
        match value {
            SamplerMode::Always => SdkSampler::AlwaysOn,
            SamplerMode::Never => SdkSampler::AlwaysOff,
            SamplerMode::Ratio(ratio) => SdkSampler::TraceIdRatioBased(ratio),
        }
    }
}

/// Initialise the OpenTelemetry framework for the process.
pub fn initialise(conf: OTelConfig, options: OTelOptions, _logger: slog::Logger) -> Result<()> {
    // OpenTelemetry internal logs now rely on Tokio Tracing.
    // To Change their handling I'll need to learn Tokio Tracing, which is not a priority right now.
    // TODO: reintroduce OTel logging hook when tracing is hooked in.

    // Skip further setup if tracing is not enabled.
    if !conf.enabled {
        return Ok(());
    }

    // Configure OTel Traces Exporter.
    let mut exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic();
    if let Some(endpoint) = conf.endpoint {
        exporter = exporter.with_endpoint(endpoint);
    }
    if let Some(timeout) = conf.timeout_sec {
        let timeout = std::time::Duration::from_secs(timeout);
        exporter = exporter.with_timeout(timeout);
    }
    let exporter = exporter.build()?;

    // Configure OTel Traces Provider.
    let mut provider_batch = BatchSpanProcessor::builder(exporter, opentelemetry_sdk::runtime::Tokio);
    if let Some(batch_config) = options.batch_config {
        provider_batch = provider_batch.with_batch_config(batch_config);
    }
    let provider = opentelemetry_sdk::trace::Builder::default()
        .with_span_processor(provider_batch.build())
        .with_sampler(SdkSampler::from(conf.sampling))
        .with_resource(options.resource)
        .build();
    opentelemetry::global::set_tracer_provider(provider);

    // Configure the global text map propagator for contexts to cross process boundaries.
    let trace = propagation::TraceContextPropagator::new();
    let baggage = propagation::BaggagePropagator::new();
    let propagator = TextMapCompositePropagator::new(vec![Box::new(trace), Box::new(baggage)]);
    opentelemetry::global::set_text_map_propagator(propagator);
    Ok(())
}
