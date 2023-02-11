//! Tools to implement Replicante Platform servers.
use anyhow::Result;

use crate::platform::models::ClusterDiscovery;
use crate::platform::models::NodeDeprovisionRequest;
use crate::platform::models::NodeProvisionRequest;
use crate::platform::models::NodeProvisionResponse;

mod context;
pub use self::context::DefaultContext;

/// Interface of a Platform server.
///
/// Using this trait for your Platform implementation opens it up for use in
/// composition patterns with tools provided by this framework (and possibly other crates).
///
/// The implementation MUST respect the [Platform Specification].
///
/// [Platform Specification]: https://www.replicante.io/docs/spec/main/platform/into/
pub trait IPlatform {
    /// Additional context passed to requests.
    type Context;

    /// Deprovision (terminate) a node in a cluster.
    fn deprovision(&self, context: &Self::Context, request: NodeDeprovisionRequest) -> Result<()>;

    /// List clusters on the platform.
    fn discover(&self, context: &Self::Context) -> Result<Vec<ClusterDiscovery>>;

    /// Provision (create) a new node for a cluster.
    fn provision(
        &self,
        context: &Self::Context,
        request: NodeProvisionRequest,
    ) -> Result<NodeProvisionResponse>;
}
