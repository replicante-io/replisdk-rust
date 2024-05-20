//! RepliCore platform definition objects and Platform related types.
use serde::Deserialize;
use serde::Serialize;

/// Definition of a Platform and its configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Platform {
    /// ID of the namespace the Platform is part of.
    pub ns_id: String,

    /// Namespaced identifier of the Platform.
    pub name: String,

    /// Activate/deactivate the Platform integration without removing it.
    ///
    /// When a Platform is deactivated it will NOT be used for cluster discovery
    /// and attempts to provision and deprovision nodes with it will fail.
    #[serde(default = "Platform::default_active")]
    pub active: bool,

    /// Cluster discovery configuration for the Platform.
    #[serde(default)]
    pub discovery: PlatformDiscoveryOptions,

    /// Platform connection method and options.
    #[serde(flatten)]
    pub transport: PlatformTransport,
}

impl Platform {
    /// Default activation state of [`Platform`]s when not specified.
    fn default_active() -> bool {
        true
    }
}

/// Cluster discovery configuration for the [`Platform`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformDiscoveryOptions {
    /// Interval, in seconds, between discovery runs.
    #[serde(default = "PlatformDiscoveryOptions::default_interval")]
    pub interval: i64,
}

impl Default for PlatformDiscoveryOptions {
    fn default() -> Self {
        Self {
            interval: Self::default_interval(),
        }
    }
}

impl PlatformDiscoveryOptions {
    /// Default interval between [`Platform`] discovery cycles.
    fn default_interval() -> i64 {
        300
    }
}

/// Reference to a [`Platform`] object defined on the cluster.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformRef {
    /// Namespace to look for the [`Platform`] from.
    ///
    /// When not set the namespace of the referencing entity is used.
    pub ns_id: Option<String>,

    /// Name of the [`Platform`] to reference.
    pub name: String,
}

/// Supported connection transports to [`Platform`]s.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlatformTransport {
    /// Connection to the platform is defined in URL format (with schema, host and path components).
    #[serde(rename = "url")]
    Url(PlatformTransportUrl),
}

/// Configuration options for URL directed connection to a Platform.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformTransportUrl {
    /// Base URL to reach the Platform server on.
    pub base_url: String,

    /// PEM encoded CA certificates bundle to add when validating remote certificates.
    #[serde(default)]
    pub tls_ca_bundle: Option<String>,

    /// Skip remote certificate validation.
    ///
    /// This option disables protection provided by server certificates
    /// and should only be used for testing.
    ///
    /// Consider using [`PlatformTransportUrl.tls_ca_bundle`]
    /// to validate custom remote certificate authorities.
    #[serde(default = "PlatformTransportUrl::default_tls_skip_verify")]
    pub tls_insecure_skip_verify: bool,
}

impl PlatformTransportUrl {
    /// Default value for [`PlatformTransportUrl.tls_insecure_skip_verify`] if not provided.
    fn default_tls_skip_verify() -> bool {
        false
    }
}
