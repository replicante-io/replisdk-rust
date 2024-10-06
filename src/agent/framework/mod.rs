//! Abstraction layer for the Replicante ecosystem to integrate with Stores.
//!
//! Replicante Agents are processes running along side the Stores to be managed.
//! These processes act as a standard interface between the Replicante ecosystem and
//! the many possible software that match the Store definition.
//!
//! The SDK aims to:
//!
//! - Standardise user experience: we want to enable anyone to write agents but we also want
//!   users that rely on agents from different authors to find a familiar landscape.
//! - Optimise agent development: agents share requirements, features and operational needs.
//!   The SDK aim to provide as much of that as possible so the focus can be on the Store needs.
//!
//! # Implementing Agents
//!
//! The SDK provides an [`Agent`] builder object that sets up a process to run agents.
//! For the agent to be built some required elements must be set and match expectations:
//!
//! - Process setup and configuration (such as process telemetry).
//! - Interactions with the store software with trait implementations.
//! - Registering supported builtin and custom actions.
//!
//! ## Agent Configuration
//!
//! The SDK hopes to minimise the required configuration as much as possible and provide
//! sensible defaults.
//!
//! For agents a configuration container is provided with [`AgentConf`].
//! This deserializable struct collects all framework related configuration values and
//! has a "slot" for agent specific options.
//!
//! The idea behind this is that configuration files can be deserialized into `AgentConf<C>`
//! structures that collect all the information needed by both framework and agent implementation.
//! The loaded configuration is the provided to the [`Agent::configure`] method.
//!
//! Aside from the user configuration options described above the framework expects some
//! agent specific options that implementations must provide.
//! The [`Agent::run`] method lists all the options that, if missing, cause the process to fail.
//!
//! ## Node Information
//!
//! The agent specification defines a some node information that must be provided.
//!
//! Since store specific logic is needed to generate this information the Agent SDK
//! provides a [`NodeInfo`] trait defining the information lookup interface.
//!
//! You can then provide a [`NodeInfoFactory`] implementation to the [`Agent`] builder.
//! The [`Agent`] will then use the [`NodeInfoFactory`] and its returned [`NodeInfo`]
//! to fulfil the agent specification.
//!
//! ## Implementing actions
//!
//! Action implementations are registered during process initialisation with
//! [`Agent::register_action`] or [`Agent::register_actions`].
//!
//! These methods expect [`ActionMetadata`], which is composed of required action kind
//! and handling logic, to register actions with the agent framework.
//!
//! ### Well Known actions
//!
//! The agent specification defines some actions standard actions agent may or must provide.
//! These actions have an agreed upon contract and all agents must implement to match.
//!
//! For example the `agent.replicante.io/cluster.init` action must initialise a new cluster
//! configuration on the instance it is run on.
//! What cluster initialisation means is dependent on the store but the outcome is the same for all.
//!
//! To define well known action implementations you must provide an [`ActionHandler`]
//! implementation to the correct function in the [`actions::wellknown`] module.
//! These functions return the appropriate [`ActionMetadata`] object to register the action with.
//!
//! ### Custom actions
//!
//! For all other actions provided by the agent a custom [`ActionMetadata`] object
//! must be defined by the implementation.
//!
//! Custom [`ActionMetadata`] require:
//!
//! - An action `kind`: to identify the action itself.
//!   Custom actions cannot use kinds in the `replicante.io` namespace.
//! - An action `handler`: any type implementing the [`ActionHandler`] trait.
//!
//! ## Examples
//!
//! The best examples are probably existing agents implemented with the SDK.
//! Below is a non-exclusive list of example repositories (in alphabetical order):
//!
//! - MongoDB Agent: <https://github.com/replicante-io/repliagent-mongodb>.
//!
//! Overall an agent setup may look like the snipped below.
//!
//! ```ignore
//! Agent::build()
//!     .configure(conf)
//!     .options(...)
//!     .telemetry_options(...)
//!     .node_info(info::NodeInfo::factory(...))
//!     .watch_task(background::custom_worker_task(...))
//!     .watch_task(background::store_monitor_task(...))
//!     .register_action(actions::custom(...))
//!     .register_action(crate::agent::framework::actions::wellknown::cluster::init(...))
//!     .register_action(crate::agent::framework::actions::wellknown::cluster::join(...))
//!
//!     // Once the agent is configured we can run it forever.
//!     .run()
//!     .await
//! ```
//!
//! [`ActionHandler`]: crate::agent::framework::actions::ActionHandler
//! [`ActionMetadata`]: crate::agent::framework::actions::ActionMetadata
//! [`actions::wellknown`] crate::agent::framework::actions::wellknown
mod conf;
mod info;
mod injector;
mod metrics;
mod node_id;
mod proc;
mod trace;

pub mod actions;
pub mod constants;
pub mod store;

#[cfg(test)]
mod tests;

pub use self::conf::AgentConf;
pub use self::conf::AgentOptions;
pub use self::info::NodeInfo;
pub use self::info::StoreVersionChain;
pub use self::info::StoreVersionCommand;
pub use self::info::StoreVersionCommandConf;
pub use self::info::StoreVersionCommandError;
pub use self::info::StoreVersionFile;
pub use self::info::StoreVersionFileError;
pub use self::info::StoreVersionFixed;
pub use self::info::StoreVersionStrategy;
pub use self::injector::Injector;
pub use self::node_id::detect_node_id;
pub use self::node_id::NodeIdDetectError;
pub use self::proc::Agent;
pub use self::proc::InitialiseHook;
pub use self::proc::InitialiseHookArgs;
pub use self::proc::NodeInfoFactory;
pub use self::proc::NodeInfoFactoryArgs;
