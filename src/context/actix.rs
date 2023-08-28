//! Additional [`Context`] feature to integrate with [ActixWeb](actix_web).
use std::future::Ready;

use actix_web::dev::forward_ready;
use actix_web::dev::Payload;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use actix_web::web::Data;
use actix_web::Error;
use actix_web::FromRequest;
use actix_web::HttpMessage;
use actix_web::HttpRequest;

use super::Context;
use super::ContextBuilder;

/// Derive a per-request [`Context`] and attach it to requests before they are handled.
pub struct ActixMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ActixMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        // Extract root context and optional middleware configuration.
        let root = request
            .app_data::<Data<Context>>()
            .expect("root Context not attached to actix app");
        let config = request.app_data::<Data<ContextConfig>>();

        // Derive the per-request context.
        let mut context = root.derive();
        if let Some(config) = config {
            for hook in &config.hooks {
                context = hook(context);
            }
        }

        // If open telemetry is available for this build also attach traces to logs.
        #[cfg(any(feature = "opentelemetry", feature = "opentelemetry_api"))]
        {
            let add_trace_id = config
                .as_ref()
                .map(|config| config.add_trace_id)
                .unwrap_or(true);
            if add_trace_id {
                context = context.log_trace();
            }
        }

        // Attach the derived context to the request.
        let context = context.build();
        request.extensions_mut().insert(context);

        // Proceed to the wrapped service and handle the request.
        self.service.call(request)
    }
}

/// Wrap an [`App`](actix_web::App) with a middleware that derives per-request contexts.
pub struct ActixTransform;

impl<S, B> Transform<S, ServiceRequest> for ActixTransform
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ActixMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let middleware = ActixMiddleware { service };
        std::future::ready(Ok(middleware))
    }
}

impl FromRequest for Context {
    type Error = Error;
    type Future = Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        let context = request
            .extensions()
            .get::<Context>()
            .expect("request has no context to extract")
            .clone();
        std::future::ready(Ok(context))
    }
}

/// Configuration of the per-request [`Context`] derivation process.
pub struct ContextConfig {
    #[cfg(any(feature = "opentelemetry", feature = "opentelemetry_api"))]
    add_trace_id: bool,
    hooks: Vec<Box<dyn Fn(ContextBuilder) -> ContextBuilder>>,
}

impl ContextConfig {
    /// Enable or disable adding the current trace ID to logs (if a trace ID is available).
    #[cfg(any(feature = "opentelemetry", feature = "opentelemetry_api"))]
    pub fn add_trace_id(mut self, add: bool) -> Self {
        self.add_trace_id = add;
        self
    }

    /// Customise the derived [`Context`] with the given callback.
    pub fn customise<F>(mut self, hook: F) -> Self
    where
        F: Fn(ContextBuilder) -> ContextBuilder + 'static,
    {
        self.hooks.push(Box::new(hook));
        self
    }

    /// Initialise a default configuration.
    pub fn new() -> Self {
        ContextConfig::default()
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        ContextConfig {
            add_trace_id: true,
            hooks: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::call_and_read_body_json;
    use actix_web::test::init_service;
    use actix_web::test::TestRequest;
    use actix_web::FromRequest;
    use actix_web::HttpMessage;
    use actix_web::HttpResponse;

    use super::super::Context;
    use super::ContextConfig;

    #[actix_web::get("/")]
    async fn inspect(context: Context) -> HttpResponse {
        let value = context.require::<u64>();
        HttpResponse::Ok().json(value)
    }

    #[actix_web::test]
    async fn extract_context() {
        let context = Context::fixture().derive().value(24u64).build();
        let request = TestRequest::get().to_http_request();
        request.extensions_mut().insert(context);
        let context = Context::extract(&request).await.unwrap();
        assert_eq!(24, *context.require::<u64>());
    }

    #[actix_web::test]
    async fn inject_context() {
        let root = Context::fixture().derive().value(26u64).build();
        let app = actix_web::App::new()
            .service(inspect)
            .app_data(actix_web::web::Data::new(root))
            .wrap(super::ActixTransform);
        let app = init_service(app).await;

        let request = TestRequest::get().uri("/").to_request();
        let response: u64 = call_and_read_body_json(&app, request).await;
        assert_eq!(response, 26u64);
    }

    #[actix_web::test]
    async fn inject_context_with_derives() {
        let conf = ContextConfig::default().customise(|builder| builder.value::<u64>(33));
        let root = Context::fixture().derive().value(26u64).build();
        let app = actix_web::App::new()
            .service(inspect)
            .app_data(actix_web::web::Data::new(conf))
            .app_data(actix_web::web::Data::new(root))
            .wrap(super::ActixTransform);
        let app = init_service(app).await;

        let request = TestRequest::get().uri("/").to_request();
        let response: u64 = call_and_read_body_json(&app, request).await;
        assert_eq!(response, 33u64);
    }
}
