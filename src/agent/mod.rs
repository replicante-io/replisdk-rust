//! Abstraction layer and utilities to implement Replicante Agents.
//!
//! # Agent Specification
//!
//! Agents MUST be built to match the
//! [Agent Specification](https://www.replicante.io/docs/spec/main/agent/intro/).
//!
//! The content of this module is coded to that specification.
//!
//! # Agents Implementations
//!
//! The SDK includes a framework to standardise and optimise the development of Replicante Agents.
//!
//! This is enabled by the `agent-framework` feature, after which you can reference the
//! `replisdk::agent::framework` module documentation for details.

#[cfg(feature = "agent-framework")]
pub mod framework;
#[cfg(feature = "agent-models")]
pub mod models;
