//! Agent interface to node information gathering.
//!
//! Also provides the tools to export the node information interface as specification
//! compliant API endpoints.
use actix_web::dev::AppService;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Data;
use actix_web::FromRequest;
use anyhow::Result;

use crate::agent::models::Node;
use crate::agent::models::ShardsInfo;
use crate::agent::models::StoreExtras;

mod node;
mod shards;
mod store_version;

#[cfg(test)]
mod tests;

pub use self::store_version::StoreVersionChain;
pub use self::store_version::StoreVersionCommand;
pub use self::store_version::StoreVersionCommandConf;
pub use self::store_version::StoreVersionCommandError;
pub use self::store_version::StoreVersionFile;
pub use self::store_version::StoreVersionFileError;
pub use self::store_version::StoreVersionFixed;
pub use self::store_version::StoreVersionStrategy;

/// Registers an [`NodeInfo`] implementation as an [`actix_web`] service.
#[derive(Clone, Debug)]
pub struct ActixServiceFactory<I>
where
    I: NodeInfo,
    I::Context: FromRequest,
{
    /// The [`slog::Logger`] usable to make [`DefaultContext`](super::DefaultContext) instances.
    logger: slog::Logger,

    /// The [`NodeInfo`] instance to register endpoints for.
    node_info: I,
}

impl<I> HttpServiceFactory for ActixServiceFactory<I>
where
    I: NodeInfo,
    I::Context: FromRequest,
{
    fn register(self, config: &mut AppService) {
        let scope = actix_web::web::scope("/info")
            .app_data(Data::new(self.logger))
            .app_data(Data::new(self.node_info))
            .service(
                actix_web::web::resource("/node")
                    .guard(actix_web::guard::Get())
                    .to(node::info_node::<I>),
            )
            .service(
                actix_web::web::resource("/shards")
                    .guard(actix_web::guard::Get())
                    .to(shards::info_shards::<I>),
            )
            .service(
                actix_web::web::resource("/store")
                    .guard(actix_web::guard::Get())
                    .to(node::info_store::<I>),
            );
        scope.register(config)
    }
}

/// Interface for Agents to get specification-defined information from a Store.
#[async_trait::async_trait]
pub trait NodeInfo: Clone + Send + Sync + 'static {
    /// Additional context passed to requests.
    type Context;

    /// Obtain information about the node, even when the store is not running.
    async fn node_info(&self, context: &Self::Context) -> Result<Node>;

    /// Obtain information about all shards managed by the node.
    async fn shards(&self, context: &Self::Context) -> Result<ShardsInfo>;

    /// Obtain information about the node only available when the store process is healthy.
    async fn store_info(&self, context: &Self::Context) -> Result<StoreExtras>;
}

/// Wrap an [`NodeInfo`] type into an [`actix_web`] service factory.
///
/// The resulting factory can be used to attach agent info endpoints onto an [`actix_web::App`].
/// The attached endpoints implement the information portion of [Agent API Specification].
///
/// [Agent API Specification]: https://www.replicante.io/docs/spec/main/agent/api/
pub fn into_actix_service<I>(node_info: I, logger: slog::Logger) -> ActixServiceFactory<I>
where
    I: NodeInfo,
    I::Context: FromRequest,
{
    ActixServiceFactory { node_info, logger }
}
