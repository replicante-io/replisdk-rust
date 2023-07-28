//! Dependency injection to enable easy access to Process Global resources.
use std::sync::RwLock;

use once_cell::sync::Lazy;
use slog::Logger;

use super::actions::ActionsRegistry;
use super::store::Store;

/// Singleton instance of the Process Globals container.
static GLOBAL_INJECTOR: Lazy<RwLock<Option<Injector>>> = Lazy::new(|| RwLock::new(None));

/// Container for all process global dependencies to be injected in other components.
#[derive(Clone)]
pub struct Injector {
    /// Registry of available action implementation for the agent.
    pub actions: ActionsRegistry,

    /// Global logger for the process.
    pub logger: Logger,

    /// Agent persisted store.
    pub store: Store,
}

impl Injector {
    /// Set the [`Injector`] instance for the process to fetch with [`Injector::global`].
    ///
    /// # Panics
    ///
    /// Panics if an [`Injector`] has already been set.
    pub(in crate::agent::framework) fn initialise(injector: Injector) {
        // Obtain a lock to initialise the global injector.
        let mut global_injector = GLOBAL_INJECTOR
            .write()
            .expect("GLOBAL_INJECTOR RwLock poisoned");

        // If the global injector is already initialised panic (without poisoning the lock).
        if global_injector.is_some() {
            drop(global_injector);
            panic!("global injector already initialised");
        }

        // Set the global injector for the process.
        slog::trace!(
            injector.logger,
            "Initialising Global Injector for the process"
        );
        *global_injector = Some(injector);
    }

    /// Get the globally set [`Injector`] instance.
    ///
    /// # Panics
    ///
    /// Panics if no [`Injector`] was set during process initialisation.
    pub fn global() -> Injector {
        GLOBAL_INJECTOR
            .read()
            .expect("GLOBAL_INJECTOR RwLock poisoned")
            .as_ref()
            .expect("global injector is not initialised")
            .clone()
    }

    #[cfg(test)]
    /// Initialise an injector to be used in tests.
    pub async fn fixture() -> Self {
        Self {
            actions: ActionsRegistry::build().finish(),
            logger: Logger::root(slog::Discard, slog::o!()),
            store: super::store::fixtures::store().await,
        }
    }
}
