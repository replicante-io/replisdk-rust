//! [`actix_web`] handler for shard info requests.
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::agent::framework::NodeInfo;
use crate::context::Context;
use crate::utils::actix::error::Result;

/// Calls the [`NodeInfo::shards`] implementation.
pub async fn info_shards<I>(agent: Data<I>, context: Context) -> Result<impl Responder>
where
    I: NodeInfo,
{
    let node = agent.shards(&context).await?;
    Ok(HttpResponse::Ok().json(node))
}
