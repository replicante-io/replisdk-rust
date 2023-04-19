//! Utilities to run [`IPlatform`s](super::IPlatform) in [`actix_web`] servers.
use actix_web::dev::AppService;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Data;
use actix_web::FromRequest;
use anyhow::Result;

use super::IPlatform;
use crate::platform::models::ClusterDefinitionNodeGroup;
use crate::platform::models::NodeProvisionRequest;

mod deprovision;
mod discover;
mod provision;

#[cfg(test)]
mod tests;

/// Wrap an [`IPlatform`](super::IPlatform) type into an [`actix_web`] service factory.
///
/// The resulting factory can be used to attach platform endpoints onto an [`actix_web::App`].
/// The attached endpoints implement the [Platform API Specification].
///
/// [Platform API Specification]: https://www.replicante.io/docs/spec/main/platform/api/
pub fn into_actix_service<P>(platform: P, logger: slog::Logger) -> ActixServiceFactory<P>
where
    P: IPlatform,
    P::Context: FromRequest,
{
    ActixServiceFactory { logger, platform }
}

/// Registers an [`IPlatform`] implementation as an [`actix_web`] service.
pub struct ActixServiceFactory<P>
where
    P: IPlatform,
    P::Context: FromRequest,
{
    /// The [`slog::Logger`] usable to make [`DefaultContext`](super::DefaultContext) instances.
    logger: slog::Logger,

    /// The [`IPlatform`] instance to register endpoints for.
    platform: P,
}

impl<P> HttpServiceFactory for ActixServiceFactory<P>
where
    P: IPlatform,
    P::Context: FromRequest,
{
    fn register(self, config: &mut AppService) {
        let scope = actix_web::web::scope("")
            .app_data(Data::new(self.logger))
            .app_data(Data::new(self.platform))
            .service(
                actix_web::web::resource("/deprovision")
                    .guard(actix_web::guard::Post())
                    .to(deprovision::deprovision::<P>),
            )
            .service(
                actix_web::web::resource("/discover")
                    .guard(actix_web::guard::Get())
                    .to(discover::discover::<P>),
            )
            .service(
                actix_web::web::resource("/provision")
                    .guard(actix_web::guard::Post())
                    .to(provision::provision::<P>),
            );
        scope.register(config)
    }
}

/// Shared logic to process [`NodeDeprovisionRequest`] models while handling provision requests.
///
/// [`NodeDeprovisionRequest`]: crate::platform::models::NodeDeprovisionRequest
pub trait NodeProvisionRequestExt {
    /// Return the [`ClusterDefinitionNodeGroup`] to provision.
    ///
    /// This variant will clone the [`ClusterDefinitionNodeGroup`] from the cluster definition
    /// so the model is left unchanged at the expense of performance.
    ///
    /// Errors if the requested group is not defined.
    fn resolve_node_group_clone(&self) -> Result<ClusterDefinitionNodeGroup>;

    /// Return the [`ClusterDefinitionNodeGroup`] to provision.
    ///
    /// This variant will remove the [`ClusterDefinitionNodeGroup`] from the cluster definition
    /// to avoid performance penalties at the expense of changing the model.
    ///
    /// Errors if the requested group is not defined.
    fn resolve_node_group_remove(&mut self) -> Result<ClusterDefinitionNodeGroup>;
}

impl NodeProvisionRequestExt for NodeProvisionRequest {
    fn resolve_node_group_clone(&self) -> Result<ClusterDefinitionNodeGroup> {
        if let Some(node_group) = self.cluster.nodes.get(&self.provision.node_group_id) {
            return Ok(node_group.clone());
        }

        let error = anyhow::anyhow!("provision.node_group_id is not defined in cluster.nodes");
        let response = serde_json::json!({
            "defined_node_groups": self.cluster.nodes.keys().collect::<Vec<&String>>(),
            "error_msg": error.to_string(),
            "node_group_id": self.provision.node_group_id,
        });
        let error = crate::utils::actix::error::Error::from(error).use_strategy(response);
        anyhow::bail!(error);
    }

    fn resolve_node_group_remove(&mut self) -> Result<ClusterDefinitionNodeGroup> {
        if let Some(node_group) = self.cluster.nodes.remove(&self.provision.node_group_id) {
            return Ok(node_group);
        }

        let error = anyhow::anyhow!("provision.node_group_id is not defined in cluster.nodes");
        let response = serde_json::json!({
            "defined_node_groups": self.cluster.nodes.keys().collect::<Vec<&String>>(),
            "error_msg": error.to_string(),
            "node_group_id": self.provision.node_group_id,
        });
        let error = crate::utils::actix::error::Error::from(error).use_strategy(response);
        anyhow::bail!(error);
    }
}
