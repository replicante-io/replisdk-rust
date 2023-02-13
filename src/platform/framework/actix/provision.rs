//! [`actix_web`] handler for node provisioning requests.
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::FromRequest;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::platform::framework::IPlatform;
use crate::platform::models::NodeProvisionRequest;
use crate::utils::actix::error::Result;

/// Encode and decode API request and response for [`IPlatform`] discovery implementation.
pub async fn provision<P>(
    payload: Json<NodeProvisionRequest>,
    platform: Data<P>,
    context: P::Context,
) -> Result<impl Responder>
where
    P: IPlatform,
    P::Context: FromRequest,
{
    let payload = payload.into_inner();
    let response = platform.provision(&context, payload)?;
    Ok(HttpResponse::Ok().json(response))
}
