//! Fixtures for Agent SDK tests.
use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::web::Data;
use actix_web::App;
use actix_web::Error;

/// Basic ActixWeb [`App`] preconfigured for unit tests.
pub fn actix_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        Error = Error,
        InitError = (),
    >,
> {
    let context = crate::context::Context::fixture();
    App::new()
        .app_data(Data::new(context))
        .wrap(crate::context::ActixTransform)
}
