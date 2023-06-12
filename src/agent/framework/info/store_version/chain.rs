//! Chain multiple [`StoreVersionStrategy`] to detect store version with the first success one.
use std::sync::Arc;

use anyhow::Result;

use super::StoreVersionStrategy;
use crate::agent::framework::DefaultContext;
use crate::agent::models::StoreVersion;

/// Chain multiple [`StoreVersionStrategy`] to detect store version with the first success one.
///
/// Strategies are tried in the order they are added with [`StoreVersionChain::strategy`].
/// If all Strategies fail the error is the fist strategy failures.
#[derive(Clone, Default)]
pub struct StoreVersionChain {
    strategies: Vec<Arc<dyn StoreVersionStrategy + Send + Sync>>,
}

impl StoreVersionChain {
    /// Add a strategy to the chain.
    pub fn strategy<S>(mut self, strategy: S) -> Self
    where
        S: StoreVersionStrategy + Send + Sync + 'static,
    {
        self.strategies.push(Arc::new(strategy));
        self
    }
}

#[async_trait::async_trait]
impl StoreVersionStrategy for StoreVersionChain {
    async fn version(&self, context: &DefaultContext) -> Result<StoreVersion> {
        let mut strategies = self.strategies.iter();
        let first = match strategies.next() {
            None => anyhow::bail!(StoreVersionChainError::NoStrategiesSet),
            Some(first) => first,
        };

        // Try the first strategy and keep track of its error in case all strategies fail.
        let error = match first.version(context).await {
            Err(error) => error,
            Ok(version) => return Ok(version),
        };

        // Try all other strategies until one succeeds.
        for strategy in strategies {
            match strategy.version(context).await {
                Ok(version) => return Ok(version),
                Err(error) => {
                    // TODO(anyhow-log-utils): Attach error as structured KV.
                    slog::debug!(
                        context.logger, "Store version detection failed";
                        "error" => ?error,
                    );
                }
            }
        }

        // Fail the chain with the first error if no strategy works.
        Err(error)
    }
}

impl std::fmt::Debug for StoreVersionChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreVersionChain")
            .field("strategies", &"[Arc<dyn StoreVersionStrategy>]".to_string())
            .finish()
    }
}

/// Errors encountered while detecting the store version from a chain.
#[derive(Debug, thiserror::Error)]
pub enum StoreVersionChainError {
    /// No store version detection strategies configured.
    #[error("no store version detection strategies configured")]
    NoStrategiesSet,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::super::StoreVersionFixed;
    use super::StoreVersionChain;
    use super::StoreVersionChainError;
    use super::StoreVersionStrategy;
    use crate::agent::framework::DefaultContext;
    use crate::agent::models::StoreVersion;

    /// Fail requests for store versions.
    struct Fail(Box<dyn Fn() -> anyhow::Error + Send + Sync>);

    impl Fail {
        /// Create a failing strategy with the error function.
        fn new<F>(f: F) -> Fail
        where
            F: Fn() -> anyhow::Error + Send + Sync + 'static,
        {
            Fail(Box::new(f))
        }
    }

    #[async_trait::async_trait]
    impl StoreVersionStrategy for Fail {
        async fn version(&self, _: &DefaultContext) -> Result<StoreVersion> {
            let error = (self.0)();
            Err(error)
        }
    }

    #[tokio::test]
    async fn first_strategy() {
        let fixed = StoreVersionFixed::new(StoreVersion {
            checkout: None,
            number: "1.2.3".into(),
            extra: None,
        });
        let chain = StoreVersionChain::default().strategy(fixed);

        let context = DefaultContext::fixture();
        let version = chain.version(&context).await.unwrap();
        assert_eq!(version.checkout, None);
        assert_eq!(version.extra, None);
        assert_eq!(version.number, "1.2.3".to_string());
    }

    #[tokio::test]
    async fn many_strategies_all_fail() {
        let chain = StoreVersionChain::default();

        let error = Fail::new(|| anyhow::anyhow!("error 1"));
        let chain = chain.strategy(error);
        let error = Fail::new(|| anyhow::anyhow!("error 2"));
        let chain = chain.strategy(error);

        let context = DefaultContext::fixture();
        let version = chain.version(&context).await;
        match version {
            Ok(version) => panic!("unexpected version {:?}", version),
            Err(_) => (),
        }
    }

    #[tokio::test]
    async fn no_strategies_defined() {
        let chain = StoreVersionChain::default();
        let context = DefaultContext::fixture();
        let version = chain.version(&context).await;
        match version {
            Ok(version) => panic!("unexpected version {:?}", version),
            Err(error) if error.is::<StoreVersionChainError>() => (),
            Err(error) => panic!("unexpected error {:?}", error),
        }
    }

    #[tokio::test]
    async fn second_strategy_success() {
        let chain = StoreVersionChain::default();

        let error = Fail::new(|| anyhow::anyhow!("error 1"));
        let chain = chain.strategy(error);

        let fixed = StoreVersionFixed::new(StoreVersion {
            checkout: None,
            number: "1.2.3".into(),
            extra: None,
        });
        let chain = chain.strategy(fixed);

        let context = DefaultContext::fixture();
        let version = chain.version(&context).await.unwrap();
        assert_eq!(version.checkout, None);
        assert_eq!(version.extra, None);
        assert_eq!(version.number, "1.2.3".to_string());
    }
}
