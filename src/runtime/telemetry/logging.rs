//! Logging related telemetry logic.
use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use slog::Drain;

/// Type erased Drain trait object for the builder to use.
type ErasedDrain = Arc<dyn slog::SendSyncRefUnwindSafeDrain<Ok = (), Err = slog::Never>>;

/// Build a new root logger for the process.
pub struct LogBuilder {
    drain: ErasedDrain,
    level: LogLevel,
    levels: BTreeMap<String, LogLevel>,
}

impl LogBuilder {
    /// Build a root logger that will emit JSON lines to the given stream.
    pub fn json<W>(stream: W, with_async: bool) -> LogBuilder
    where
        W: std::io::Write + Send + 'static,
    {
        let drain = slog_json::Json::new(stream)
            .add_default_keys()
            .build()
            .ignore_res();

        // Skip the Mutex synchronisation if slog_async is in use.
        let drain: ErasedDrain = if with_async {
            let drain = slog_async::Async::new(drain).build().ignore_res();
            Arc::new(drain)

        // Otherwise use a Mutex to synchronise a shared drain.
        } else {
            let drain = std::sync::Mutex::new(drain).ignore_res();
            Arc::new(drain)
        };

        LogBuilder {
            drain,
            level: Default::default(),
            levels: Default::default(),
        }
    }

    /// Build a root logger that will emit formatted lines to the terminal.
    pub fn term(with_async: bool) -> LogBuilder {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().ignore_res();

        // Skip the Mutex synchronisation if slog_async is in use.
        let drain: ErasedDrain = if with_async {
            let drain = slog_async::Async::new(drain).build().ignore_res();
            Arc::new(drain)

        // Otherwise use a Mutex to synchronise a shared drain.
        } else {
            let drain = std::sync::Mutex::new(drain).ignore_res();
            Arc::new(drain)
        };

        LogBuilder {
            drain,
            level: Default::default(),
            levels: Default::default(),
        }
    }

    /// Complete logger initialisation and returns a root logger.
    pub fn finish(self) -> slog::Logger {
        // Configure log level filtering using slog-envlogger.
        let drain = if std::env::var("RUST_LOG").is_ok() {
            slog_envlogger::new(self.drain)
        } else {
            let mut builder =
                slog_envlogger::LogBuilder::new(self.drain).filter(None, self.level.into());
            for (prefix, level) in self.levels {
                builder = builder.filter(Some(&prefix), level.into());
            }
            builder.build()
        };

        // Attach global extra information and create root logger.
        let values = slog::o!(
            "module" => slog::FnValue(|record : &slog::Record| record.module()),
        );
        slog::Logger::root(drain, values)
    }

    /// Configure the default logging level for the process.
    pub fn level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Extend logging level configuration for a collection of module prefixes.
    pub fn levels(mut self, levels: BTreeMap<String, LogLevel>) -> Self {
        self.levels.extend(levels);
        self
    }
}

/// Configuration option for process logging.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LogConfig {
    /// Default logging level for the process.
    ///
    /// This option can be overridden for specific modules with the `levels` map.
    ///
    /// This option is ignored if `RUST_LOG` is configured.
    #[serde(default)]
    pub level: LogLevel,

    /// Logging levels for specific modules.
    ///
    /// Module prefixes are taken into account, with longer prefixes overriding their parents.
    #[serde(default)]
    pub levels: BTreeMap<String, LogLevel>,

    /// Asynchronously emit log events.
    ///
    /// Asynchronous logging can improve performance but can result in some loss
    /// if the process exists abruptly.
    #[serde(rename = "async", default = "LogConfig::default_log_async")]
    pub log_async: bool,

    /// How logs are emitted.
    #[serde(default)]
    pub mode: LogMode,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: Default::default(),
            levels: Default::default(),
            log_async: LogConfig::default_log_async(),
            mode: Default::default(),
        }
    }
}

impl LogConfig {
    /// Default value for the `log_async` config option.
    fn default_log_async() -> bool {
        true
    }
}

/// Possible log event levels.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Emit critical log events only.
    #[serde(alias = "CRITICAL", alias = "critical")]
    Critical,

    /// Emit error or more sever log events.
    #[serde(alias = "ERROR", alias = "error")]
    Error,

    /// Emit warning or more sever log events.
    #[default]
    #[serde(alias = "WARNING", alias = "warning")]
    Warning,

    /// Emit information or more sever log events.
    #[serde(alias = "INFO", alias = "info")]
    Info,

    /// Emit debug or more sever log events.
    #[serde(alias = "DEBUG", alias = "debug")]
    Debug,

    /// Emit trace or more sever log events.
    #[serde(alias = "TRACE", alias = "trace")]
    Trace,
}

impl From<LogLevel> for slog::FilterLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Critical => slog::FilterLevel::Critical,
            LogLevel::Error => slog::FilterLevel::Error,
            LogLevel::Warning => slog::FilterLevel::Warning,
            LogLevel::Info => slog::FilterLevel::Info,
            LogLevel::Debug => slog::FilterLevel::Debug,
            LogLevel::Trace => slog::FilterLevel::Trace,
        }
    }
}

/// Supported logging formats and destinations.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum LogMode {
    /// Format logs as a stream of JSON encoded lines to standard out.
    #[default]
    Json,

    /// Display logs onto a terminal, with option colour support.
    Terminal,
}

/// Programmatic options for logging.
pub struct LogOptions {
    /// Forward events from the pseudo-standard `log` crate to the root logger.
    pub capture_log_crate: bool,
}

impl Default for LogOptions {
    fn default() -> Self {
        LogOptions {
            capture_log_crate: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LogBuilder;

    #[test]
    fn log_to_json_async() {
        let line = Vec::new();
        let builder = LogBuilder::json(line, true);
        let logger = builder.finish();
        slog::info!(logger, "test"; "key" => "value");
    }

    #[test]
    fn log_to_json_sync() {
        let line = Vec::new();
        let builder = LogBuilder::json(line, false);
        let logger = builder.finish();
        slog::info!(logger, "test"; "key" => "value");
    }
}
