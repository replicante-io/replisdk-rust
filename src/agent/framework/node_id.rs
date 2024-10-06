//! Reusable logic to detect the node ID for the agent.
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use slog::Logger;

use crate::agent::framework::constants::ENV_NODE_ID;
use crate::agent::framework::AgentConf;

/// Errors encountered while attempting to detect the node ID.
#[derive(Debug, thiserror::Error)]
pub enum NodeIdDetectError {
    /// Node ID detection could not find the ID.
    #[error(
        "Node ID detection could not find the ID, try setting {} or the configuration",
        ENV_NODE_ID
    )]
    NotFound,
}

/// Attempts to detect the Node ID for the system.
///
/// The Node ID is looked up from different places, with the first valid result used:
///
/// 1. Check the environment variable defined by `ENV_NODE_ID`.
/// 2. Check the configuration file.
pub async fn detect_node_id<C>(conf: &AgentConf<C>, logger: &Logger) -> Result<String>
where
    C: Clone + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
{
    slog::debug!(logger, "Node ID detection started");
    // Attempt to use the ENV_NODE_ID environment variable.
    if let Ok(node_id) = std::env::var(ENV_NODE_ID) {
        slog::info!(
            logger,
            "Detected Node ID from environment variable {}", ENV_NODE_ID;
            "node_id" => &node_id,
        );
        return Ok(node_id);
    }

    // Attempt to use the configuration.
    if let Some(node_id) = conf.node_id.clone() {
        slog::info!(
            logger, "Detected Node ID from configuration";
            "node_id" => &node_id,
        );
        return Ok(node_id);
    }

    // Exhausted detection methods before a node ID was found.
    anyhow::bail!(NodeIdDetectError::NotFound);
}

#[cfg(test)]
mod tests {
    use slog::Logger;

    use super::detect_node_id;
    use super::NodeIdDetectError;
    use crate::agent::framework::AgentConf;

    fn fixtures() -> (AgentConf<()>, Logger) {
        let conf = AgentConf::<()>::default();
        let logger = Logger::root(slog::Discard, slog::o!());
        (conf, logger)
    }

    // NOTE: environment variable test changes the whole process and breaks other tests.

    #[tokio::test]
    async fn node_id_from_conf() {
        let (mut conf, logger) = fixtures();
        conf.node_id = Some("node.id.456".into());
        let node_id = detect_node_id(&conf, &logger).await.unwrap();
        assert_eq!(node_id, "node.id.456");
    }

    #[tokio::test]
    async fn node_id_not_found() {
        let (conf, logger) = fixtures();
        let node_id = detect_node_id(&conf, &logger).await;
        match node_id {
            Err(error) if error.is::<NodeIdDetectError>() => {
                match error.downcast::<NodeIdDetectError>().unwrap() {
                    NodeIdDetectError::NotFound => (),
                }
            }
            other => panic!("unexpected node id {:?}", other),
        };
    }
}
