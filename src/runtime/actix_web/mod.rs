//! Actix Web server runtime configuration utilities.
use std::fmt::Debug;
use std::sync::Arc;

use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware::Compress;
use actix_web::middleware::Condition;
use actix_web::web::ServiceConfig;
use actix_web::App;
use actix_web::Error;
use prometheus::Registry;

use crate::utils::actix::metrics::MetricsCollector;
use crate::utils::actix::metrics::MetricsExporter;

mod conf;

pub use self::conf::ServerConfig;

type ConfCallback = Arc<dyn Fn(&mut ServiceConfig) + Send + Sync + 'static>;

/// Dynamic collection of configuration logic for [`actix_web::App`]s.
#[derive(Clone, Default)]
pub struct AppConfigurer {
    configs: Vec<ConfCallback>,
}

impl AppConfigurer {
    /// Configure an [`actix_web::App`] by appling closures using its [`ServiceConfig`].
    pub fn configure(&self, app: &mut ServiceConfig) {
        for call in &self.configs {
            call(app);
        }
    }

    /// Append a configuration closure to the collection.
    pub fn with_config<F>(&mut self, config: F) -> &mut Self
    where
        F: Fn(&mut ServiceConfig) + Send + Sync + 'static,
    {
        let config = Arc::new(config);
        self.configs.push(config);
        self
    }
}

/// Factory pattern to simplify creation of [`actix_web::App`].
///
/// This factory is intended for use with [`HttpServer`](actix_web::HttpServer) initialisation code:
///
/// 1. The `AppFactory` is created (and configured) once before `HttpServer` initialisation.
/// 2. Create an `HttPServer` with a closure that captures the `AppFactory`.
/// 3. Initialise an `App` with [`AppFactory::initialise`].
/// 4. Apply any customisations you want, including from `AppFactory` "opt-ins".
/// 5. Finalise the `App` with [`AppFactory::finalise`].
///
/// ```ignore
/// // Initialise the ActixWeb HttPServer with an app factory.
/// let factory = AppFactory::configure()
///     // Configure the AppFactory as needed.
///     .done();
/// let server = HttpServer::new(|| {
///     let app = factory.initialise();
///     // Customise the app here as desired.
///     factory.finalise(app)
/// });
///
/// // Configure and run the server.
/// let conf = ServerConfig::default();
/// let server = conf.apply(server)?;
/// server.run();
/// ```
#[derive(Clone)]
pub struct AppFactory {
    app_conf: AppConfigurer,
    conf: ServerConfig,
    metrics_collector: MetricsCollector,
    metrics_exporter: MetricsExporter,
    metrics_path: &'static str,
}

impl AppFactory {
    /// Begin configuration of an [`AppFactory`].
    pub fn configure(app_conf: AppConfigurer, conf: ServerConfig) -> AppFactoryBuilder {
        AppFactoryBuilder {
            app_conf,
            conf,
            metrics_path: "/metrics",
            metrics_prefix: None,
            metrics_registry: None,
        }
    }

    /// Initialise an [`actix_web::App`] with defaults and provided customisations.
    ///
    /// The following customisations are applied:
    ///
    /// - All customisations defined in the [`AppConfigurer`] are applied.
    pub fn initialise(
        &self,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            Error = Error,
            InitError = (),
        >,
    > {
        App::new().configure(|app| self.app_conf.configure(app))
    }

    /// Finalise the [`actix_web::App`] with middleware to wrap every request.
    ///
    /// The following middleware are applied:
    ///
    /// - User configurable request/response de/compression.
    /// - Request metrics collection.
    /// - Request logging.
    /// - Request tracing.
    ///
    /// The following customisations are also applied:
    ///
    /// - Endpoint to expose metrics in prometheus format.
    pub fn finalise<B, T>(
        &self,
        app: App<T>,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            Error = Error,
            InitError = (),
        >,
    >
    where
        B: MessageBody + 'static,
        T: ServiceFactory<
                ServiceRequest,
                Response = ServiceResponse<B>,
                Config = (),
                Error = Error,
                InitError = (),
            > + 'static,
    {
        // Configure format for request logging.
        let logger = match &self.conf.log_format {
            None => actix_web::middleware::Logger::default(),
            Some(format) => actix_web::middleware::Logger::new(format),
        };

        // Define endpoint for metrics export.
        let metrics_exporter = self.metrics_exporter.clone();
        let metrics_endpoint = actix_web::web::resource(self.metrics_path)
            .route(actix_web::web::get().to(metrics_exporter));

        app.service(metrics_endpoint)
            .wrap(Condition::new(
                self.conf.compress_responses,
                Compress::default(),
            ))
            .wrap(self.metrics_collector.clone())
            .wrap(logger)
            .wrap(actix_web_opentelemetry::RequestTracing::new())
    }
}

/// Builder pattern for [`AppFactory`] instances.
#[derive(Clone)]
pub struct AppFactoryBuilder {
    app_conf: AppConfigurer,
    conf: ServerConfig,
    metrics_path: &'static str,
    metrics_prefix: Option<&'static str>,
    metrics_registry: Option<prometheus::Registry>,
}

impl AppFactoryBuilder {
    /// Complete [`AppFactory`] configuration and validate provided options.
    pub fn done(self) -> AppFactory {
        // Validate the builder.
        let metrics_prefix = self
            .metrics_prefix
            .expect("prefix for metrics names MUST be provided");
        let metrics_registry = self
            .metrics_registry
            .expect("registry for metrics MUST be provided");

        // Prepare metrics collection middleware and report endpoint.
        let metrics_exporter = MetricsExporter::new(metrics_registry.clone());
        let metrics_collector = MetricsCollector::build()
            .prefix(metrics_prefix)
            .registry(metrics_registry)
            .finish();

        // Return the factory that can initialise and finalise Apps.
        AppFactory {
            app_conf: self.app_conf,
            conf: self.conf,
            metrics_collector,
            metrics_exporter,
            metrics_path: self.metrics_path,
        }
    }

    /// Provide the required request metrics parameters.
    pub fn metrics(mut self, prefix: &'static str, registry: Registry) -> Self {
        self.metrics_prefix = Some(prefix);
        self.metrics_registry = Some(registry);
        self
    }

    /// Set the endpoint path to export metrics on.
    pub fn metrics_path(mut self, path: &'static str) -> Self {
        self.metrics_path = path;
        self
    }
}

/// Errors encountered while building an [`HttpServer`](actix_web::HttpServer).
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// Unable to binding the server to an address.
    ///
    /// Error parameters:
    ///
    /// - The address the bind failed for.
    #[error("unable to bind the server to '{0}'")]
    Bind(String),

    /// Unable to set client CA certificates from file.
    ///
    /// Error parameters:
    ///
    /// - The path to the client CA bundle file.
    #[error("unable to set client CA certificates from file '{0}")]
    TlsClientCAs(String),

    /// Unable to initialise TLS engine.
    ///
    /// Error parameters:
    ///
    /// - The TLS engine that was used (for example: openssl).
    #[error("unable to initialise {0} TLS engine")]
    TlsInit(&'static str),

    /// Unable to set server certificate from PEM file.
    ///
    /// Error parameters:
    ///
    /// - The path to the server certificate file.
    #[error("unable to set server certificate from PEM file '{0}")]
    TlsServerCert(String),

    /// Unable to set server private key from PEM file.
    ///
    /// Error parameters:
    ///
    /// - The path to the server private key file.
    #[error("unable to set server private key from PEM file '{0}")]
    TlsServerKey(String),
}
