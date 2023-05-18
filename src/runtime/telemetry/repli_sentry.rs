//! Sentry initialisation related logic.
use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use sentry::ClientInitGuard;
use serde::Deserialize;
use serde::Serialize;

/// Initialise the Sentry framework for the process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SentryConfig {
    /// Sentry DSN (Data Source Name) to send events to.
    #[serde(default)]
    pub dsn: Option<String>,

    /// Enable sentry integration for the process.
    #[serde(default = "SentryConfig::default_enabled")]
    pub enabled: bool,

    /// The ratio of generated events that are submitted to Sentry (between 0.0 and 1.0).
    #[serde(default = "SentryConfig::default_sample_ratio")]
    pub sample_ratio: f32,

    /// Maximum delay in seconds to process shutdown to flush pending events to Sentry.
    #[serde(default = "SentryConfig::default_shutdown_timeout")]
    pub shutdown_timeout: u64,
}

impl SentryConfig {
    fn default_enabled() -> bool {
        false
    }

    fn default_sample_ratio() -> f32 {
        1.0
    }

    fn default_shutdown_timeout() -> u64 {
        2
    }
}

impl Default for SentryConfig {
    fn default() -> Self {
        SentryConfig {
            dsn: None,
            enabled: Self::default_enabled(),
            sample_ratio: Self::default_sample_ratio(),
            shutdown_timeout: Self::default_shutdown_timeout(),
        }
    }
}

/// Programmatic options for the Sentry framework.
pub struct SentryOptions {
    /// List of module prefixes Sentry consider "not in app" when processing backtraces.
    pub in_app_exclude: Vec<&'static str>,

    /// List of module prefixes Sentry considers "in app" when processing backtraces.
    pub in_app_include: Vec<&'static str>,

    /// The release tag to attach to all events sent to Sentry.
    pub release: std::borrow::Cow<'static, str>,
}

impl SentryOptions {
    /// Default sentry options with the given release.
    pub fn for_release<S>(release: S) -> SentryOptions
    where
        S: Into<std::borrow::Cow<'static, str>>,
    {
        SentryOptions {
            in_app_exclude: Default::default(),
            in_app_include: Default::default(),
            release: release.into(),
        }
    }
}

/// Errors initialising sentry for the process.
#[derive(Debug, thiserror::Error)]
pub enum SentryError {
    /// Error returned when the configured DSN is not valid.
    #[error("the configured DSN is not valid")]
    InvalidDsn,

    /// Error returned when the configured sample ration is outside the valid range.
    #[error("the sampling ratio must be between 0 and 1")]
    InvalidSampleRatio,
}

/// Initialise the Sentry framework for the process.
pub fn initialise(conf: SentryConfig, options: SentryOptions) -> Result<Option<ClientInitGuard>> {
    if !conf.enabled {
        return Ok(None);
    }

    // Validated configuration.
    let dsn = conf
        .dsn
        .map(|dsn| sentry::types::Dsn::from_str(&dsn).context(SentryError::InvalidDsn))
        .transpose()?;
    if conf.sample_ratio < 0.0 || conf.sample_ratio > 1.0 {
        anyhow::bail!(SentryError::InvalidSampleRatio);
    }

    // Prepare the sentry client configuration.
    let mut in_app_include = options.in_app_include;
    in_app_include.push("replisdk");
    in_app_include.push("replisdk_experimental");
    let options = sentry::ClientOptions {
        dsn,
        in_app_exclude: options.in_app_exclude,
        in_app_include,
        release: Some(options.release),
        sample_rate: conf.sample_ratio,
        shutdown_timeout: std::time::Duration::from_secs(conf.shutdown_timeout),
        ..Default::default()
    };
    let guard = sentry::init(options);
    Ok(Some(guard))
}

#[cfg(test)]
mod tests {
    use super::SentryConfig;
    use super::SentryError;
    use super::SentryOptions;

    #[test]
    fn dsn_not_valid() {
        let conf = SentryConfig {
            dsn: Some("invalid-dsn".into()),
            enabled: true,
            ..Default::default()
        };
        let opts = SentryOptions::for_release("replisdk-telemetry-tests@0.0.0");
        match super::initialise(conf, opts) {
            Ok(_) => panic!("sentry should not have initialised"),
            Err(error) if error.is::<SentryError>() => {
                let error = error.downcast_ref::<SentryError>().unwrap();
                assert!(
                    matches!(error, SentryError::InvalidDsn),
                    "unexpected SentryError variant",
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }
    }

    #[test]
    fn sample_ratio_above_1() {
        let conf = SentryConfig {
            enabled: true,
            sample_ratio: 1.001,
            ..Default::default()
        };
        let opts = SentryOptions::for_release("replisdk-telemetry-tests@0.0.0");
        match super::initialise(conf, opts) {
            Ok(_) => panic!("sentry should not have initialised"),
            Err(error) if error.is::<SentryError>() => {
                let error = error.downcast_ref::<SentryError>().unwrap();
                assert!(
                    matches!(error, SentryError::InvalidSampleRatio),
                    "unexpected SentryError variant",
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }
    }

    #[test]
    fn sample_ratio_below_0() {
        let conf = SentryConfig {
            enabled: true,
            sample_ratio: -0.1,
            ..Default::default()
        };
        let opts = SentryOptions::for_release("replisdk-telemetry-tests@0.0.0");
        match super::initialise(conf, opts) {
            Ok(_) => panic!("sentry should not have initialised"),
            Err(error) if error.is::<SentryError>() => {
                let error = error.downcast_ref::<SentryError>().unwrap();
                assert!(
                    matches!(error, SentryError::InvalidSampleRatio),
                    "unexpected SentryError variant",
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }
    }

    #[test]
    fn sentry_not_configured() {
        let conf = SentryConfig::default();
        let opts = SentryOptions::for_release("replisdk-telemetry-tests@0.0.0");
        let guard = super::initialise(conf, opts)
            .expect("sentry init can't fail when sentry is not enabled");
        assert!(guard.is_none(), "sentry is enabled somehow");
    }
}
