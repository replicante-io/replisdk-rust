//! Abstraction layer for Replicante ecosystem to integrate with (infrastructure) Platforms.
//!
//! # Platform integrations
//!
//! Platforms are an abstraction to allow Replicante Core (and other parts of the ecosystem)
//! to integrate with the infrastructure provisioning layer.
//!
//! In this context infrastructure provisioning layer refers to platforms able to manage nodes
//! such as physical infrastructure, cloud platforms, container orchestrators, and similar.
//!
//! The Platform abstraction implements the following interface to infrastructures:
//!
//! * Cluster discovery.
//! * Node provisioning.
//! * Node deprovisioning.
//!
//! ## Cluster discovery
//!
//! Cluster discovery abstracts listing of nodes running on the Platform,
//! grouping together nodes that belong to the same cluster.
//!
//! Cluster discovery returns information as described by [`models::ClusterDiscovery`] records.
//!
//! ## Node provisioning
//!
//! Node provisioning is still a work in progress and will be added soon.
//!
//! ## Node deprovisioning
//!
//! Node deprovisioning is still a work in progress and will be added soon.
#[cfg(any(feature = "platform-models"))]
pub mod models;
