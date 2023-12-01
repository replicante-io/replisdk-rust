//! RepliCore namespace definition objects.
use serde::Deserialize;
use serde::Serialize;

/// Namespace Level defaults for TLS client connections to resources in the cluster.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TlsDefaults {
    /// Optional PEM formatted bundle of CA certificates to validate remote servers.
    #[serde(default)]
    pub ca_bundle: Option<String>,

    // TODO: add client_key_secret once secrets storage is solved.
}

/// Definition of a Namespace and its configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Namespace {
    /// Identifier of the namespace (also known as the namespace name).
    pub id: String,

    /// Default TLS options used when connecting to resources in the namespace.
    #[serde(default)]
    pub tls: TlsDefaults,

    /// Lifecycle status of the namespace.
    #[serde(default)]
    pub status: NamespaceStatus,
}

/// Possible lifecycle states a namespace can be in.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum NamespaceStatus {
    /// The namespace and its objects are fully managed.
    #[default]
    Active,

    /// The namespace and its objects are ignored.
    Inactive,

    /// The namespace and its clusters are monitored but actions are forbidden.
    Observed,

    /// THe deletion of the namespace objects was requested and is in progress.
    ///
    /// For example clusters in the namespace are being deprovisioned.
    Deleting,

    /// The deletion of the namespace objects is complete.
    ///
    /// The namespace itself can be deleted at any time.
    Deleted,
}
