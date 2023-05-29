use std::fmt::Debug;

use actix_http::Request;
use actix_http::Response;
use actix_service::IntoServiceFactory;
use actix_web::body::MessageBody;
use actix_web::dev::AppConfig;
use actix_web::dev::ServiceFactory;
use actix_web::Error;
use actix_web::HttpServer;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::AppConfigurer;
use super::BuildError;
use super::OpinionatedBuilder;

/// User focused configuration options for [`HttpServer`]s.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Sets the maximum number of pending connections.
    ///
    /// Refer to [`HttpServer::backlog`] for more details.
    #[serde(default)]
    pub backlog: Option<u32>,

    /// Resolves socket address(es) and binds server to created listener(s).
    ///
    /// Refer to [`HttpServer::bind`] for more details.
    #[serde(default = "ServerConfig::default_bind")]
    pub bind: String,

    // TODO: client_disconnect_timeout: Option<u64/millisec>
    // TODO: client_request_timeout: Option<u64/millisec>
    // TODO: keep_alive: Option<???>
    /// Format of server access logs.
    #[serde(default)]
    pub log_format: Option<String>,
    // TODO: log_format: Option<String>,
    // TODO: max_connections: Option<usize>
    // TODO? server_hostname - should this be an option instead?
    // TODO: shutdown_timeout: Option<u64/sec>
    // TODO: tls_handshake_timeout: Option<u64/millisec>
    // TODO: tls_options: {enabled, ???}
    // TODO: workers: Option<usize>
}

impl ServerConfig {
    fn default_bind() -> String {
        "localhost:6000".into()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            backlog: Default::default(),
            bind: Self::default_bind(),
            log_format: None,
        }
    }
}

impl ServerConfig {
    /// Apply the configuration object itself to a [`HttpServer`].
    pub fn apply<F, I, S, B>(self, server: HttpServer<F, I, S, B>) -> Result<HttpServer<F, I, S, B>>
    where
        F: Fn() -> I + Send + Clone + 'static,
        I: IntoServiceFactory<S, Request>,
        S: ServiceFactory<Request, Config = AppConfig> + 'static,
        S::Error: Into<Error>,
        S::InitError: Debug,
        S::Response: Into<Response<B>>,
        B: MessageBody + 'static,
    {
        let mut server = server;
        if let Some(backlog) = self.backlog {
            server = server.backlog(backlog);
        }
        server = server
            .bind(&self.bind)
            .with_context(|| BuildError::Bind(self.bind))?
            .disable_signals();
        Ok(server)
    }

    /// Build a [`Server`] instance with the standard configuration applied.
    ///
    /// The focus of opinionated builds is to avoid as much logic as possible in applications.
    /// To achieve this, choices are made in the builder that may limit otherwise possible options.
    pub fn opinionated(self, app: AppConfigurer) -> OpinionatedBuilder {
        OpinionatedBuilder {
            app,
            conf: self,
            metrics_path: "/metrics",
            metrics_prefix: None,
            metrics_registry: None,
        }
    }
}
