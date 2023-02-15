//! [`actix_web`] handler for node deprovision requests.
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::FromRequest;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::platform::framework::IPlatform;
use crate::platform::models::NodeDeprovisionRequest;
use crate::utils::actix::error::Result;

/// Decode a node deprovision request and calls the [`IPlatform`] implementation.
pub async fn deprovision<P>(
    payload: Json<NodeDeprovisionRequest>,
    platform: Data<P>,
    context: P::Context,
) -> Result<impl Responder>
where
    P: IPlatform,
    P::Context: FromRequest,
{
    let payload = payload.into_inner();
    platform.deprovision(&context, payload).await?;
    Ok(HttpResponse::NoContent())
}
