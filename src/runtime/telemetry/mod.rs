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
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

mod logging;
mod opentel;

pub use self::logging::LogBuilder;
pub use self::logging::LogConfig;
pub use self::logging::LogLevel;
pub use self::logging::LogMode;
pub use self::logging::LogOptions;
pub use self::opentel::OTelConfig;
pub use self::opentel::OTelOptions;

/// Configured telemetry resources.
///
/// Internally also tracks "initialisation guards" for telemetry components.
/// Dropping this container while the process is running can result in unexpected behaviours.
pub struct Telemetry {
    /// Root logger for the process.
    pub logger: slog::Logger,

    // Initialisation guards for global scopes.
    #[allow(dead_code)]
    slog_scope_guard: Option<slog_scope::GlobalLoggerGuard>,
}

/// Telemetry configuration options.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Logging configuration for the process.
    pub logs: LogConfig,

    /// OpenTelemetry configuration for the process.
    pub otel: OTelConfig,
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
}

/// Initialise telemetry for the process.
pub async fn initialise(conf: TelemetryConfig, options: TelemetryOptions) -> Result<Telemetry> {
    let (logger, slog_scope_guard) = initialise_logger(conf.logs, options.logs);
    self::opentel::initialise(conf.otel, options.otel)?;
    Ok(Telemetry {
        logger,
        slog_scope_guard,
    })
}

/// Initialise a root logger based on the provided configuration.
pub fn initialise_logger(
    conf: LogConfig,
    options: LogOptions,
) -> (slog::Logger, Option<slog_scope::GlobalLoggerGuard>) {
    // Build the root logger first.
    let builder = match conf.mode {
        LogMode::Json => LogBuilder::json(std::io::stdout(), conf.log_async),
        LogMode::Terminal => LogBuilder::term(conf.log_async),
    };
    let logger = builder.level(conf.level).levels(conf.levels).finish();

    // Initialise slog_scope and slog_stdlog libraries if `log` capture is desired.
    let mut slog_scope_guard = None;
    if options.capture_log_crate {
        let guard = slog_scope::set_global_logger(logger.clone());
        slog_stdlog::init().expect("capture of log crate initialisation failed");
        slog_scope_guard = Some(guard);
    }

    // Return the root logger.
    (logger, slog_scope_guard)
}
