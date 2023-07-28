//! Agent actions execution and interfaces.
//!
//! Also provides the tools to export the actions interface as specification
//! compliant API endpoints.

mod api;
mod executor;
mod handler;
mod registry;

pub(in crate::agent::framework) use executor::ActionsExecutor;
pub(in crate::agent::framework) use handler::ActionHandlerChangeValue;

pub use api::ActionsService;
pub use handler::ActionHandler;
pub use handler::ActionHandlerChanges;
pub use registry::ActionMetadata;
pub use registry::ActionMetadataBuilder;
pub use registry::ActionNotFound;
pub use registry::ActionsRegistry;
pub use registry::ActionsRegistryBuilder;
