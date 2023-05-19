//! Prometheus metrics initialisation related logic.
use std::collections::BTreeMap;

use anyhow::Context;
use anyhow::Result;
use prometheus::Registry;
use serde::Deserialize;
use serde::Serialize;

/// Configuration of Prometheus metrics collection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrometheusConfig {
    /// Additional labels to attach to all process metrics.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,

    /// Enable or disable collecting process-level metrics (linux only).
    #[serde(default = "PrometheusConfig::default_process_metrics")]
    pub process_metrics: bool,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        PrometheusConfig {
            labels: Default::default(),
            process_metrics: PrometheusConfig::default_process_metrics(),
        }
    }
}

impl PrometheusConfig {
    fn default_process_metrics() -> bool {
        true
    }
}

/// Errors initialising the Prometheus registry for the process.
#[derive(Debug, thiserror::Error)]
pub enum PrometheusError {
    /// Returned when the configured global labels are not valid.
    #[error("invalid prometheus global labels found in the configuration")]
    InvalidLabels,
}

/// Initialise a Prometheus metrics registry for the process.
pub fn initialise(conf: PrometheusConfig) -> Result<Registry> {
    // Create the registry with globally configured labels.
    let labels = if conf.labels.is_empty() {
        None
    } else {
        Some(conf.labels.into_iter().collect())
    };
    let reg = Registry::new_custom(None, labels).context(PrometheusError::InvalidLabels)?;

    // If configured and supported enable process level metrics.
    #[cfg(target_os = "linux")]
    {
        if conf.process_metrics {
            let proc = prometheus::process_collector::ProcessCollector::for_self();
            let _ = reg.register(Box::new(proc));
        }
    }

    Ok(reg)
}

#[cfg(test)]
mod tests {
    use super::PrometheusConfig;

    #[test]
    fn global_labels_added() {
        let conf = PrometheusConfig {
            labels: [("test".into(), "value".into())].into_iter().collect(),
            ..Default::default()
        };
        let reg = super::initialise(conf).expect("prometheus to registry initialise");
        let counter = prometheus::Counter::new("test_metrics", "test metric")
            .expect("unable to create test metric");
        reg.register(Box::new(counter))
            .expect("unable to register test metric");
        let metric = reg
            .gather()
            .iter()
            .next()
            .and_then(|family| {
                let metric = family.get_metric();
                metric.into_iter().next()
            })
            .cloned();
        let metric = metric.expect("no test metric gathered");
        let label = metric
            .get_label()
            .into_iter()
            .next()
            .expect("label to be exported");
        assert_eq!(label.get_name(), "test");
        assert_eq!(label.get_value(), "value");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn skip_process_metrics() {
        let conf = PrometheusConfig {
            process_metrics: false,
            ..Default::default()
        };
        let reg = super::initialise(conf).expect("prometheus registry to initialise");
        let metrics = reg.gather();
        assert_eq!(metrics.len(), 0);
    }
}
