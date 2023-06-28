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
use openssl::ssl::SslAcceptor;
use openssl::ssl::SslVerifyMode;
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

    /// Maximum time in milliseconds allowed for clients to send all request headers.
    ///
    /// If a client takes longer to transmit all request headers the request is failed.
    ///
    /// A value of zero disables the timeout.
    #[serde(default)]
    pub client_request_timeout: Option<u64>,

    /// Enable response compression, if supported by clients.
    ///
    /// The compression method is negotiated with the client using the `Accept-Encoding` header.
    #[serde(default = "ServerConfig::default_compress_responses")]
    pub compress_responses: bool,

    /// Server preference for how long to keep connections alive when idle.
    ///
    /// A value of zero disables keep alive and connections will be
    /// closed immediately after the response is sent.
    #[serde(default)]
    pub keep_alive: Option<u64>,

    /// Format of server access logs.
    #[serde(default)]
    pub log_format: Option<String>,

    /// Maximum number of concurrent connections for each server worker.
    ///
    /// This option is available for both TLS and non-TLS modes due to the greatly
    /// different CPU requirements.
    ///
    /// Once the limit is reach listening sockets will stop accepting connections
    /// until currently open connections are closed.
    #[serde(default)]
    pub max_connections: Option<usize>,

    /// Maximum number of concurrent TLS connections for each server worker.
    ///
    /// This option is available for both TLS and non-TLS modes due to the greatly
    /// different CPU requirements.
    ///
    /// Once the limit is reach listening sockets will stop accepting connections
    /// until currently open connections are closed.
    #[serde(default)]
    pub max_connections_tls: Option<usize>,

    /// Time in seconds workers are given to complete requests in progress when a shutdown
    /// signal is received.
    #[serde(default)]
    pub shutdown_timeout: Option<u64>,

    /// Configure the server to run with TLS encryption.
    #[serde(default)]
    pub tls: Option<ServerConfigTls>,

    /// Number of workers handling HTTP requests.
    pub workers: Option<usize>,
}

impl ServerConfig {
    fn default_bind() -> String {
        "localhost:8000".into()
    }

    fn default_compress_responses() -> bool {
        true
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            backlog: Default::default(),
            bind: Self::default_bind(),
            client_request_timeout: None,
            compress_responses: true,
            keep_alive: None,
            log_format: None,
            max_connections: None,
            max_connections_tls: None,
            shutdown_timeout: None,
            tls: None,
            workers: None,
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
        let mut server = server.disable_signals();
        if let Some(backlog) = self.backlog {
            server = server.backlog(backlog);
        }
        if let Some(timeout) = self.client_request_timeout {
            let timeout = std::time::Duration::from_millis(timeout);
            server = server.client_request_timeout(timeout);
        }
        if let Some(keep_alive) = self.keep_alive {
            let keep_alive = std::time::Duration::from_millis(keep_alive);
            server = server.keep_alive(keep_alive);
        }
        if let Some(max) = self.max_connections {
            server = server.max_connections(max);
        }
        if let Some(max) = self.max_connections_tls {
            server = server.max_connection_rate(max);
        }
        if let Some(timeout) = self.shutdown_timeout {
            server = server.shutdown_timeout(timeout);
        }
        if let Some(workers) = self.workers {
            server = server.workers(workers);
        }

        // Bind the server, with TLS if configured.
        let server = match self.tls {
            Some(tls) if tls.enabled => {
                let mut engine = SslAcceptor::mozilla_modern_v5(openssl::ssl::SslMethod::tls())
                    .context(BuildError::TlsInit("openssl"))?;
                engine
                    .set_certificate_file(&tls.server_private_cert, openssl::ssl::SslFiletype::PEM)
                    .with_context(|| BuildError::TlsServerCert(tls.server_private_cert))?;
                engine
                    .set_private_key_file(&tls.server_private_key, openssl::ssl::SslFiletype::PEM)
                    .with_context(|| BuildError::TlsServerKey(tls.server_private_key))?;

                if let Some(bundle) = tls.client_ca_bundle {
                    engine
                        .set_ca_file(&bundle)
                        .with_context(|| BuildError::TlsClientCAs(bundle))?;
                    engine.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
                }
                if let Some(timeout) = tls.handshake_timeout {
                    let timeout = std::time::Duration::from_millis(timeout);
                    server = server.tls_handshake_timeout(timeout);
                }
                server.bind_openssl(&self.bind, engine)
            }
            _ => server.bind(&self.bind),
        };
        let server = server.with_context(|| BuildError::Bind(self.bind))?;
        Ok(server)
    }

    /// Build a [`Server`] instance with the standard configuration applied.
    ///
    /// The focus of opinionated builds is to avoid as much logic as possible in applications.
    /// To achieve this, choices are made in the builder that may limit otherwise possible options.
    ///
    /// [`Server`]: actix_web::dev::Server
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

/// Configure the server to run with TLS encryption.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServerConfigTls {
    /// Path to a PEM bundle of Certificate Authorities to verify client certificates with.
    ///
    /// When this option is set, clients MUST provide a certificate that is valid.
    #[serde(default)]
    pub client_ca_bundle: Option<String>,

    /// Enable TLS for the server.
    #[serde(default = "ServerConfigTls::default_enabled")]
    pub enabled: bool,

    /// Maximum time in milliseconds a TLS handshake must complete in.
    ///
    /// If the handshake does not complete in time the connection is closed.
    #[serde(default)]
    pub handshake_timeout: Option<u64>,

    /// Path to the PEM encoded server private certificate file.
    pub server_private_cert: String,

    /// Path to the PEM encoded server private key file.
    pub server_private_key: String,
}

impl ServerConfigTls {
    fn default_enabled() -> bool {
        true
    }
}
