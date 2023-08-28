//! [`actix_web`] handler for node info requests.
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::agent::framework::NodeInfo;
use crate::context::Context;
use crate::utils::actix::error::Result;

/// Calls the [`NodeInfo::node_info`] implementation.
pub async fn info_node<I>(agent: Data<I>, context: Context) -> Result<impl Responder>
where
    I: NodeInfo,
{
    let node = agent.node_info(&context).await?;
    Ok(HttpResponse::Ok().json(node))
}

/// Calls the [`NodeInfo::store_info`] implementation.
pub async fn info_store<I>(agent: Data<I>, context: Context) -> Result<impl Responder>
where
    I: NodeInfo,
{
    let node = agent.store_info(&context).await?;
    Ok(HttpResponse::Ok().json(node))
}
