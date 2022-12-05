//! Public types and shared logic to integrate, extend or interact with Replicante Core.
//!
//! # Replicante Core
//!
//! The Replicante project is a composition of different processes and systems that
//! work together to manage data stores.
//!
//! Replicante Core is the control plane at the center of the ecosystem. It observes [`Platform`]s
//! and `Agent`s and coordinate activities based on user configurations and requests.
#[cfg(feature = "replicore-models")]
pub mod models;
