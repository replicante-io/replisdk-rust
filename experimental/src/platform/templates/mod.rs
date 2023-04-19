//! Utilities to manage store node templates in Replicante Platform servers.
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

mod lookup;

pub use self::lookup::TemplateLookup;

/// Cluster node context to render templates with.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Additional attributes attached to all nodes in the cluster.
    ///
    /// These attributes can be used by the template to customise nodes in the cluster.
    pub attributes: Map<String, Value>,

    /// ID of the cluster to add the node to.
    pub cluster_id: String,

    /// The store software to provision on the node.
    pub store: String,

    /// The version of the store software to provision on the node.
    pub store_version: String,
}

/// Load templates from disk and prepares them for rendering.
///
/// Loading of templates includes initialisation of the templating engine with all extra
/// helpers, application globals and such possible features.
///
/// # Experimental Properties
///
/// - Should `Template` have trait constraints?
/// - Should `Template` be `Clone`?
///   - Could limit implementations.
///   - But would allow caching `TemplateFactory` decorators and such.
///     - Could still do with generic type constraints instead of `Clone` super-trait?
#[async_trait::async_trait]
pub trait TemplateFactory: Send + Sync {
    /// Type of templates returned by this factory.
    type Template;

    /// Load a template from disk.
    async fn load(&self, options: &TemplateLoadOptions) -> Result<Self::Template>;
}

/// Manifest options passed to [`TemplateFactory`] when loading templates.
#[derive(Clone, Debug)]
pub struct TemplateLoadOptions {
    /// Manifest options - allowed values depend on the [`TemplateFactory`] getting them.
    pub options: serde_json::Value,

    /// Path to the template or templates to load.
    pub template: String,
}
