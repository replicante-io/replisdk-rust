//! Utilities to run [`IPlatform`s](super::IPlatform) in [`actix_web`] servers.
use actix_web::dev::AppService;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Data;
use actix_web::FromRequest;

use super::IPlatform;

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
