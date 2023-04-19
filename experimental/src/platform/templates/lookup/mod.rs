//! Template lookup logic
use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use semver::VersionReq;
use serde::Deserialize;

mod manifests;
#[cfg(test)]
mod tests;

use super::TemplateContext;
use super::TemplateLoadOptions;
use crate::platform::templates::TemplateFactory;

/// Checks if the attributes match the matchers.
fn attributes_match(
    attributes: &serde_json::Map<String, serde_json::Value>,
    matchers: &HashMap<String, Value>,
) -> bool {
    for (name, value) in matchers {
        let is_match = attributes
            .get(name)
            .map(|attribute| value == attribute)
            .unwrap_or(false);
        if !is_match {
            return false;
        }
    }
    true
}

/// Errors looking up templates, loading lookup manifests, etc ...
#[derive(Debug, thiserror::Error)]
pub enum LookupError {
    #[error("invalid path to manifest file {0}")]
    // (path,)
    InvalidManifestPath(String),

    #[error("invalid semantic version requirement in manifest {0}")]
    // (path,)
    InvalidVersionRequirement(String),
}

impl LookupError {
    /// The path to a manifest file is not valid.
    fn invalid_manifest_path<P: Into<String>>(path: P) -> Self {
        Self::InvalidManifestPath(path.into())
    }

    /// Manifest includes an invalid version requirement string.
    fn invalid_version_requirement<P: Into<String>>(path: P) -> Self {
        Self::InvalidVersionRequirement(path.into())
    }
}

/// Rule to select the store to lookup the version from.
pub struct StoreRule {
    /// Values that must match the attributes from the lookup request to select this store.
    pub matchers: HashMap<String, Value>,

    /// ID of the store that must match the lookup request to select this store.
    pub store: String,

    /// List of [`VersionRule`]s to use when this store is matched.
    pub versions: Vec<VersionRule>,
}

/// Loaded manifest(s) to lookup a specific template for a store and its version.
pub struct TemplateLookup<T: TemplateFactory> {
    /// Instance of the [`TemplateFactory`] to load templates with.
    factory: T,

    /// List of [`StoreRule`]s to select a store with.
    stores: Vec<StoreRule>,
}

impl<T: TemplateFactory> TemplateLookup<T> {
    /// Load additional lookup rules from the given manifest path.
    ///
    /// The additional rules have a lower priority to any previously loaded rule.
    ///
    /// See [`TemplateLookup::load_file`] for details on the format of manifest files.
    pub async fn extend_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // Find the root of all relative paths in the manifest.
        let path = path.as_ref();
        let root = path
            .parent()
            .ok_or_else(|| LookupError::invalid_manifest_path(path.to_string_lossy()))?;

        // Load the stores manifest.
        let manifest = std::fs::File::open(path)
            .with_context(|| LookupError::invalid_manifest_path(path.to_string_lossy()))?;
        tokio::task::yield_now().await;
        let manifest: self::manifests::StoresManifest = serde_yaml::from_reader(manifest)?;

        // Iterate over the manifest.
        let mut stores = Vec::new();
        for rule in manifest.stores {
            let mut store = StoreRule {
                matchers: rule.matchers,
                store: rule.store,
                versions: Vec::new(),
            };

            // Load the versions manifest for this specific store.
            let path = root.join(rule.manifest);
            let manifest = std::fs::File::open(&path)
                .with_context(|| LookupError::invalid_manifest_path(path.to_string_lossy()))?;
            tokio::task::yield_now().await;
            let manifest: self::manifests::VersionsManifest = serde_yaml::from_reader(manifest)?;

            for rule in manifest.versions {
                let version = VersionReq::parse(&rule.version).with_context(|| {
                    LookupError::invalid_version_requirement(path.to_string_lossy())
                })?;
                let mut template: TemplateLoadOptions = rule.template.into();
                let target = root.join(template.template);
                template.template = target
                    .to_str()
                    .ok_or_else(|| LookupError::invalid_manifest_path(target.to_string_lossy()))?
                    .to_string();
                store.versions.push(VersionRule {
                    matchers: rule.matchers,
                    template,
                    version,
                });
            }

            stores.push(store);
        }

        // If all manifests are valid extend the rules set.
        self.stores.extend(stores);
        Ok(())
    }

    /// Load template lookup rules from the given manifest path.
    ///
    /// # Manifest files
    ///
    /// Manifest files are indexes to identify what store the template belongs to.
    /// To enable more flexibility and co-ownership the manifests do not manage store versions
    /// directly but instead point to version manifest files.
    ///
    /// Example manifest:
    ///
    /// ```yaml
    /// ---
    /// stores:
    ///   # Lookup versions for MongoDB nodes provisioned with a `replica-set` mode attribute.
    ///   - store: mongodb
    ///     manifest: mongodb/replicas.yaml
    ///     mode: replica-set
    ///
    ///   # Lookup versions for MongoDB nodes provisioned with a `sharded` mode attribute.
    ///   - store: mongodb
    ///     manifest: mongodb/shards.yaml
    ///     mode: sharded
    ///
    ///   # Lookup versions for Kafka clusters.
    ///   - store: kafka
    ///     manifest: kafka.yaml
    /// ```
    ///
    /// ## Version manifest files
    ///
    /// Once the target store is identified these manifests define which templates
    /// should be loaded, and optionally how, depending on the requested version.
    ///
    /// ```yaml
    /// ---
    /// versions:
    ///   # Lookup templates for MongoDB Shared config Replica Set.
    ///   - version: 3.2
    ///     template: v3.2/config/*
    ///     role: config
    ///
    ///   # Lookup templates for MongoDB Shared shard Replica Set.
    ///   - version: 3.2
    ///     template: v3.2/shard/*
    ///     role: shard
    ///
    ///   # Set additional template options for nodes running MongoDB v5.0 and later.
    ///   - version: '>= 5.0'
    ///     template:
    ///       target: v6/*
    ///       options:
    ///         main_template: node.yaml
    /// ```
    pub async fn load_file<P: AsRef<Path>>(factory: T, path: P) -> Result<Self> {
        let stores = Vec::new();
        let mut lookup = TemplateLookup { factory, stores };
        lookup.extend_from_file(path).await?;
        Ok(lookup)
    }

    /// Lookup a template for a store and version.
    ///
    /// # Lookup order
    ///
    /// Stores are looked up in the order they are defined in the file(s) they are loaded from.
    /// Each store is checked in that order and the first match is taken.
    /// This means higher up definitions win.
    ///
    /// Once a store is selected the same process is applied to versions.
    /// Only versions within the selected store are considered.
    ///
    /// If no version in the selected store match the request no version is selected by the lookup.
    /// Even if a later store would have matched and a version in it would have matched too.
    ///
    /// # Attributes matching
    ///
    /// Stores and versions can be filtered on more then just a name/version range.
    /// Attributes to the node provision request are checked to select stores and versions too.
    ///
    /// Store and version rules can include additional properties that MUST match request
    /// attributes for the rule to be selected.
    ///
    /// For rules to match attributes:
    ///
    /// - If a rule has a property then the request attributes MUST have it also.
    /// - The value of a rule property MUST match the value of the corresponding attribute EXACTLY.
    /// - Any request attribute that is NOT also a rule property is ignored.
    pub async fn lookup(&self, context: &TemplateContext) -> Result<Option<T::Template>> {
        // Parse store version into a semver usable version.
        let version = semver::Version::parse(&context.store_version)?;

        // Lookup a store rule.
        let store_rule = self.stores.iter().find(|rule| {
            rule.store == context.store && attributes_match(&context.attributes, &rule.matchers)
        });
        let store_rule = match store_rule {
            None => return Ok(None),
            Some(rule) => rule,
        };

        // Lookup a version rule.
        let version_rule = store_rule.versions.iter().find(|rule| {
            rule.version.matches(&version) && attributes_match(&context.attributes, &rule.matchers)
        });
        let version_rule = match version_rule {
            None => return Ok(None),
            Some(rule) => rule,
        };

        // Load the template based on the rule.
        let template = self.factory.load(&version_rule.template).await?;
        Ok(Some(template))
    }
}

impl<T: TemplateFactory> Extend<StoreRule> for TemplateLookup<T> {
    fn extend<I: IntoIterator<Item = StoreRule>>(&mut self, iter: I) {
        self.stores.extend(iter)
    }
}

/// Subset of [`serde_json::Value`] types allowed in matchers.
#[derive(Debug, Default, Deserialize, PartialEq)]
pub enum Value {
    /// Represents a JSON boolean.
    Bool(bool),

    /// Represents a JSON null value.
    #[default]
    Null,

    /// Represents a JSON number, whether integer or floating point.
    Number(serde_json::Number),

    /// Represents a JSON string.
    String(String),
}

impl PartialEq<serde_json::Value> for Value {
    fn eq(&self, other: &serde_json::Value) -> bool {
        match (self, other) {
            (Value::Bool(me), serde_json::Value::Bool(other)) => me.eq(other),
            (Value::Null, serde_json::Value::Null) => true,
            (Value::Number(me), serde_json::Value::Number(other)) => me.eq(other),
            (Value::String(me), serde_json::Value::String(other)) => me.eq(other),
            _ => false,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<serde_json::Number> for Value {
    fn from(value: serde_json::Number) -> Self {
        Value::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&'static str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

/// Rule to select the template version to load.
pub struct VersionRule {
    /// Values that must match the attributes from the lookup request to select this version.
    pub matchers: HashMap<String, Value>,

    /// Options to load templates selected by this rule.
    pub template: TemplateLoadOptions,

    /// Semantic version requirements to select this version.
    pub version: VersionReq,
}
