//! Collection of actions defined for an [`Agent`](crate::agent::framework::Agent).
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use super::ActionHandler;

/// List of restricted action kind domains which can only be used by the SDK itself.
const REPLICANTE_DOMAINS: [&str; 1] = ["replicante.io"];

/// Metadata attached to action implementations.
#[derive(Debug)]
pub struct ActionMetadata {
    /// Identifier of the action implementation.
    pub(in crate::agent::framework) kind: String,

    /// [`ActionHandler`] to invoke for [`ActionExecution`] with matching `kind`.
    pub(in crate::agent::framework) handler: Box<dyn ActionHandler>,
}

impl ActionMetadata {
    /// Build the metadata record for an [`ActionHandler`].
    ///
    /// # Panics
    ///
    /// The method panics when attempting to build metadata for restricted kind domains.
    /// Action kinds are defined in the form `{domain}/{name}` to ensure actions with
    /// generic names don't clash with each other.
    ///
    /// Restricted domains are:
    ///
    /// - `replicante.io`
    /// - `*.replicante.io`
    pub fn build<S, H>(kind: S, handler: H) -> ActionMetadataBuilder
    where
        H: ActionHandler + 'static,
        S: Into<String>,
    {
        // Check the action kind for use of restricted domains.
        let kind = kind.into();
        let domain = kind
            .split('/')
            .next()
            .expect("split string to have at least one entry");
        for restricted in REPLICANTE_DOMAINS {
            if domain == restricted || domain.ends_with(&format!(".{}", restricted)) {
                panic!("unable to build metadata for restricted domain {}", domain);
            }
        }
        Self::build_internal(kind, handler)
    }

    /// Build the metadata record for an [`ActionHandler`] WITHOUT domain checks.
    pub(in crate::agent::framework) fn build_internal<S, H>(
        kind: S,
        handler: H,
    ) -> ActionMetadataBuilder
    where
        H: ActionHandler + 'static,
        S: Into<String>,
    {
        let kind = kind.into();
        let handler = Box::new(handler);
        ActionMetadataBuilder { kind, handler }
    }
}

/// Build the metadata record for an [`ActionHandler`].
pub struct ActionMetadataBuilder {
    kind: String,
    handler: Box<dyn ActionHandler>,
}

impl ActionMetadataBuilder {
    /// Complete the [`ActionMetadata`] build process.
    pub fn finish(self) -> ActionMetadata {
        ActionMetadata {
            kind: self.kind,
            handler: self.handler,
        }
    }
}

/// Collection of [`ActionMetadata`] records known to the agent.
#[derive(Clone, Debug)]
pub struct ActionsRegistry {
    entries: Arc<HashMap<String, ActionMetadata>>,
}

impl ActionsRegistry {
    /// Build an [`ActionsRegistry`] instance.
    pub fn build() -> ActionsRegistryBuilder {
        ActionsRegistryBuilder {
            entries: Default::default(),
        }
    }

    /// Lookup the metadata for the given action kind.
    pub fn lookup<S>(&self, kind: S) -> Result<&ActionMetadata>
    where
        S: Into<String>,
    {
        let kind = kind.into();
        self.entries
            .get(&kind)
            .ok_or(ActionNotFound { kind })
            .map_err(anyhow::Error::from)
    }
}

/// Build an [`ActionsRegistry`] instance.
pub struct ActionsRegistryBuilder {
    entries: HashMap<String, ActionMetadata>,
}

impl ActionsRegistryBuilder {
    /// Complete the [`ActionsRegistry`] build process.
    pub fn finish(self) -> ActionsRegistry {
        let entries = Arc::new(self.entries);
        ActionsRegistry { entries }
    }

    /// Register the metadata for a new action.
    pub fn register(mut self, metadata: ActionMetadata) -> Self {
        if self.entries.contains_key(&metadata.kind) {
            panic!(
                "action {} cannot be registered more then once",
                metadata.kind,
            );
        }

        let kind = metadata.kind.clone();
        self.entries.insert(kind, metadata);
        self
    }
}

/// Metadata for action not found.
#[derive(Debug, thiserror::Error)]
#[error("metadata for action {kind} not found")]
pub struct ActionNotFound {
    /// The action kind being looked up.
    pub kind: String,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::super::ActionHandlerChanges as Changes;
    use super::ActionHandler;
    use super::ActionMetadata;
    use super::ActionsRegistry;
    use crate::agent::models::ActionExecution;
    use crate::context::Context;

    #[derive(Debug)]
    struct TestNoop {}
    #[async_trait::async_trait]
    impl ActionHandler for TestNoop {
        async fn invoke(&self, _: &Context, action: &ActionExecution) -> Result<Changes> {
            Ok(Changes::to(action.state.phase))
        }
    }

    #[rstest::rstest]
    #[case("replicante.io/test")]
    #[case("replicante.io/with/many/splits")]
    #[case("agent.replicante.io/test")]
    #[case("core.replicante.io/test")]
    #[should_panic(expected = "unable to build metadata for restricted domain")]
    fn metadata_build_reject_domains(#[case] kind: &str) {
        let handler = TestNoop {};
        ActionMetadata::build(kind, handler);
    }

    #[rstest::rstest]
    #[case("example.com/test")]
    #[case("no-domain.test")]
    #[case("action/with/many/splits")]
    fn metadata_build_allow_domains(#[case] kind: &str) {
        let handler = TestNoop {};
        let metadata = ActionMetadata::build(kind, handler).finish();
        assert_eq!(metadata.kind, kind);
    }

    #[test]
    fn lookup_action() {
        let handler = TestNoop {};
        let metadata = ActionMetadata::build("test", handler).finish();
        let registry = ActionsRegistry::build().register(metadata).finish();
        registry.lookup("test").unwrap();
    }

    #[test]
    fn lookup_action_not_found() {
        let registry = ActionsRegistry::build().finish();
        let error = registry.lookup("test").unwrap_err();
        let error: super::ActionNotFound = error.downcast().unwrap();
        assert_eq!(error.kind, "test");
    }

    #[test]
    #[should_panic(expected = "action test cannot be registered more then once")]
    fn register_action_twice() {
        let handler = TestNoop {};
        let metadata = ActionMetadata::build("test", handler).finish();
        let registry = ActionsRegistry::build().register(metadata);

        let handler = TestNoop {};
        let metadata = ActionMetadata::build("test", handler).finish();
        registry.register(metadata);
    }
}
