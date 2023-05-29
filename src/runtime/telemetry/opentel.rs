//! OpenTelemetry initialisation related logic.
use anyhow::Result;
use opentelemetry::sdk::trace::Sampler as SdkSampler;
use opentelemetry_otlp::WithExportConfig;
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
    pub batch_config: Option<opentelemetry::sdk::trace::BatchConfig>,

    /// Attributes representing the process that produces telemetry data.
    pub resource: Option<opentelemetry::sdk::Resource>,
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
pub fn initialise(conf: OTelConfig, options: OTelOptions) -> Result<()> {
    if !conf.enabled {
        return Ok(());
    }

    // Create and configure OTel Exporter.
    let mut exporter = opentelemetry_otlp::new_exporter().tonic();
    if let Some(endpoint) = conf.endpoint {
        exporter = exporter.with_endpoint(endpoint);
    }
    if let Some(timeout) = conf.timeout_sec {
        let timeout = std::time::Duration::from_secs(timeout);
        exporter = exporter.with_timeout(timeout);
    }

    // Create and configure OTel Pipeline.
    let mut pipeline_conf =
        opentelemetry::sdk::trace::config().with_sampler(SdkSampler::from(conf.sampling));
    if let Some(resource) = options.resource {
        pipeline_conf = pipeline_conf.with_resource(resource);
    }
    let mut pipeline = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(pipeline_conf);
    if let Some(batch_config) = options.batch_config {
        pipeline = pipeline.with_batch_config(batch_config);
    }
    pipeline.install_batch(opentelemetry::runtime::Tokio)?;
    Ok(())
}
