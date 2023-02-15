//! Tools to implement Replicante Platform servers.
use anyhow::Result;

use crate::platform::models::ClusterDiscoveryResponse;
use crate::platform::models::NodeDeprovisionRequest;
use crate::platform::models::NodeProvisionRequest;
use crate::platform::models::NodeProvisionResponse;

mod context;
pub use self::context::DefaultContext;

#[cfg(feature = "platform-framework_actix")]
mod actix;
#[cfg(feature = "platform-framework_actix")]
pub use {self::actix::into_actix_service, self::actix::ActixServiceFactory};

/// Interface of a Platform server.
///
/// Using this trait for your Platform implementation opens it up for use in
/// composition patterns with tools provided by this framework (and possibly other crates).
///
/// The implementation MUST respect the [Platform Specification].
///
/// [Platform Specification]: https://www.replicante.io/docs/spec/main/platform/into/
#[async_trait::async_trait]
pub trait IPlatform: 'static {
    /// Additional context passed to requests.
    type Context;

    /// Deprovision (terminate) a node in a cluster.
    async fn deprovision(
        &self,
        context: &Self::Context,
        request: NodeDeprovisionRequest,
    ) -> Result<()>;

    /// List clusters on the platform.
    async fn discover(&self, context: &Self::Context) -> Result<ClusterDiscoveryResponse>;

    /// Provision (create) a new node for a cluster.
    async fn provision(
        &self,
        context: &Self::Context,
        request: NodeProvisionRequest,
    ) -> Result<NodeProvisionResponse>;
}
