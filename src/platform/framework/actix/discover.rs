//! [`actix_web`] handler for cluster discovery requests.
use actix_web::web::Data;
use actix_web::FromRequest;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::platform::framework::IPlatform;
use crate::utils::actix::error::Result;

/// Call the [`IPlatform`] cluster discovery implementation and encode the response.
pub async fn discover<P>(platform: Data<P>, context: P::Context) -> Result<impl Responder>
where
    P: IPlatform,
    P::Context: FromRequest,
{
    let response = platform.discover(&context)?;
    Ok(HttpResponse::Ok().json(response))
}
