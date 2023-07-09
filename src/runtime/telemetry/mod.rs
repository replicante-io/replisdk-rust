//! Utilities to initialise runtime telemetry of the process.
//!
//! # Logging
//!
//! The telemetry runtime utilities provide shared configuration and initialisation code
//! for process logging.
//! The initialisation process will return an [`slog::Logger`] instance configured as specified.
//!
//! Logging configuration options for end users are defined by the [`LogConfig`] object.
//! This object implements [`serde`] `Serialize` and `Deserialize` traits so application
//! developers can compose it into their configuration files.
//!
//! Additional customisation options are defined in the ['LogOptions`] object.
//! These options are for application developers to tune process logging to their preferences.
//!
//! ## Async Logging
//!
//! Users should be aware that asynchronous logging can provide performance improvements
//! but can also lead to loss of logs if the process exists before logs are flushed.
//!
//! ## Capturing `log` events
//!
//! This is provided by the [`slog-stdlog`] crate which defines a `log` backend that
//! can send events to `slog::Logger`.
//!
//! Because [`slog-stdlog`] requires [`slog-scope`] to be configured, that library is also
//! configured when `log` capturing is enabled.
//!
//! # OpenTelemetry
//!
//! [OpenTelemetry](https://opentelemetry.io/) is a framework for processes to generate
//! telemetry data in an integrated and portable way.
//!
//! The project aim is to support integrated generation of logs, metrics and tracing data
//! but at this time only tracing data is supported by these utilities.
//!
//! When enabled, the telemetry data can be exported in one of the following formats.
//! The protocol, as well as its exporter options, can be configured at runtime.
//!
//! - Open Telemetry Protocol (OTLP): export data in the OpenTelemetry native protocol.
//!
//! ## Configuration
//!
//! By default OpenTelemetry is NOT enabled, which means all generated data is discarded.
//! When enabled, telemetry data is exported to a locally running OpenTelemetry agent.
//!
//! Additional user configuration options can be provided with [`OTelConfig`]
//! and applications can tune the OpenTelemetry integration with [`OTelOptions`].
//!
//! # Prometheus Metrics
//!
//! The [Prometheus](https://prometheus.io/) metrics integration provides a
//! [`Registry`](prometheus::Registry) for the process to attach generated metrics
//! that can then be exported.
//!
//! On Linux systems, this integration can also register a set of process wide metrics.
//!
//! ## Prometheus vs OpenTelemetry
//!
//! Prometheus is used to generate and export metrics instead of OpenTelemetry
//! because metrics support in OpenTelemetry for Rust is still subject to major changes
//! (at the time of writing).
//!
//! # Sentry
//!
//! [Sentry](https://sentry.io/) is an error and events reporting solution to collect
//! data from applications and understand when and where issues occur faster.
//!
//! The Sentry client is not automatically enabled and users must configure it.
//! Enabling sentry is a two steps process:
//!
//! 1. Enable process integration by setting [`SentryConfig::enabled`] to `true`.
//! 2. Configure a Sentry DSN (Data Source Name) for the client to emit events.
//!
//! ## Configuration
//!
//! User specified configuration options are set in the [`SentryConfig`] struct
//! similarly to the other telemetry frameworks.
//!
//! Unlike other cases Sentry has some required options that the application MUST
//! provide during initialisation. These are set in the [`SentryOptions`] struct.
//!
//! - [`SentryOptions::release`]: the identifier of the current application and version.
//!   You can create a `SentryOptions` struct from a release with [`SentryOptions::for_release`].
//!   For example, you can get a static release name with:
//!   ```ignore
//!   const RELEASE_ID: &str = concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"));
//!   ```
//! - [`SentryOptions::in_app_include`]: a list of module prefixes for Sentry to consider part
//!   of the instrumented applications.
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

mod logging;
mod opentel;
mod prom;
mod repli_sentry;

pub use self::logging::LogBuilder;
pub use self::logging::LogConfig;
pub use self::logging::LogLevel;
pub use self::logging::LogMode;
pub use self::logging::LogOptions;
pub use self::opentel::OTelConfig;
pub use self::opentel::OTelOptions;
pub use self::prom::PrometheusConfig;
pub use self::prom::PrometheusError;
pub use self::repli_sentry::SentryConfig;
pub use self::repli_sentry::SentryError;
pub use self::repli_sentry::SentryOptions;

/// Configured telemetry resources.
///
/// Internally also tracks "initialisation guards" for telemetry components.
/// Dropping this container while the process is running can result in unexpected behaviours.
pub struct Telemetry {
    /// Root logger for the process.
    pub logger: slog::Logger,

    /// Registry for the process to attach Prometheus metrics to.
    pub metrics: prometheus::Registry,

    // Initialisation guards for global scopes.
    #[allow(dead_code)]
    sentry: Option<sentry::ClientInitGuard>,

    #[allow(dead_code)]
    slog_scope_guard: self::logging::StdLogSafeGuard,
}

/// Telemetry configuration options.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Logging configuration for the process.
    #[serde(default)]
    pub logs: LogConfig,

    /// OpenTelemetry configuration for the process.
    #[serde(default)]
    pub otel: OTelConfig,

    /// Configuration for Prometheus metrics generated by the process.
    #[serde(default)]
    pub prom_metrics: PrometheusConfig,

    /// Sentry configuration for the process.
    #[serde(default)]
    pub sentry: SentryConfig,
}

/// Programmatic telemetry options.
///
/// Where config options are intended for user/runtime configuration,
/// programmatic options are intended to process developers to tune their runtime.
pub struct TelemetryOptions {
    /// Logging programmatic options.
    pub logs: LogOptions,

    /// OpenTelemetry programmatic options.
    pub otel: OTelOptions,

    /// Sentry programmatic options.
    pub sentry: SentryOptions,
}

impl TelemetryOptions {
    /// Default telemetry options using the given Sentry release ID.
    pub fn for_sentry_release<S>(release: S) -> TelemetryOptions
    where
        S: Into<std::borrow::Cow<'static, str>>,
    {
        TelemetryOptions {
            logs: Default::default(),
            otel: Default::default(),
            sentry: SentryOptions::for_release(release),
        }
    }
}

/// Initialise telemetry for the process.
pub async fn initialise(conf: TelemetryConfig, options: TelemetryOptions) -> Result<Telemetry> {
    let (logger, slog_scope_guard) = self::logging::initialise(conf.logs, options.logs);
    self::opentel::initialise(conf.otel, options.otel)?;
    let sentry = self::repli_sentry::initialise(conf.sentry, options.sentry)?;
    let metrics = self::prom::initialise(conf.prom_metrics)?;
    Ok(Telemetry {
        logger,
        metrics,
        sentry,
        slog_scope_guard,
    })
}
