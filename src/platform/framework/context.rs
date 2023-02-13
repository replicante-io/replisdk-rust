//! Default `Context` definition for Platform framework.
use slog::Logger;

/// Default additional context for [`IPlatform`](super::IPlatform) implementations.
///
/// When using custom contexts you can still reuse the default logic by embedding this
/// struct as a field to your custom context type.
pub struct DefaultContext {
    /// Contextual logger to be used by the operation.
    pub logger: Logger,
}

#[cfg(feature = "platform-framework_actix")]
impl actix_web::FromRequest for DefaultContext {
    type Error = actix_web::Error;
    type Future = futures::future::Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let logger = match req.app_data::<actix_web::web::Data<Logger>>() {
            Some(logger) => logger.as_ref().clone(),
            None => todo!(),
        };
        futures::future::ready(Ok(DefaultContext { logger }))
    }
}
