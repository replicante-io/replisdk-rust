//! Actix Web server runtime configuration utilities.
use std::fmt::Debug;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::web::ServiceConfig;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;
use prometheus::Registry;
use slog::Logger;

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

/// Errors encountered while building an [`HttpServer`].
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// Unable to binding the server to an address.
    ///
    /// Error parameters:
    ///
    /// - The address the bind failed for.
    #[error("unable to bind the server to '{0}'")]
    Bind(String),
}

/// Build a [`Server`] instance with the standard configuration applied.
pub struct OpinionatedBuilder {
    app: AppConfigurer,
    conf: ServerConfig,
    metrics_path: &'static str,
    metrics_prefix: Option<&'static str>,
    metrics_registry: Option<prometheus::Registry>,
}

impl OpinionatedBuilder {
    /// Complete configuration and return a running server.
    ///
    /// # Panics
    ///
    /// This method panics if the builder is not given all required parameters:
    ///
    /// - A prefix for metrics names MUST be provided along side a [`Registry`].
    pub fn run(self, logger: Option<&Logger>) -> Result<Server> {
        // Validate the builder.
        let metrics_prefix = self
            .metrics_prefix
            .expect("prefix for metrics names MUST be provided");
        let metrics_registry = self
            .metrics_registry
            .expect("registry for metrics MUST be provided");

        // Grab objects to move into the app factory.
        let log_format = self.conf.log_format.clone();
        let export = MetricsExporter::new(metrics_registry.clone());
        let metrics = MetricsCollector::build()
            .prefix(metrics_prefix)
            .registry(metrics_registry)
            .finish();
        let metrics_path = self.metrics_path;

        // Create server and app instances.
        let server = HttpServer::new(move || {
            let app = App::new();
            let app = app.configure(|app| self.app.configure(app));

            // Configure request logging.
            let logger = match &log_format {
                None => actix_web::middleware::Logger::default(),
                Some(format) => actix_web::middleware::Logger::new(format),
            };
            let app = app.wrap(logger);

            // Configure request metrics collection and export.
            let metrics_res = actix_web::web::resource(metrics_path)
                .route(actix_web::web::get().to(export.clone()));
            app.service(metrics_res).wrap(metrics.clone())

            // TODO(tracing): Configure request tracing.
            //app
        });
        if let Some(logger) = logger {
            slog::info!(logger, "Starting HTTP Server bound at {}", &self.conf.bind);
        }
        let server = self.conf.apply(server)?;
        Ok(server.run())
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
