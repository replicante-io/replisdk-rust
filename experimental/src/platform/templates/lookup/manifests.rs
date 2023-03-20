//! Model definitions for [`TemplateLookup`](super::TemplateLookup) manifests on disk.
use std::collections::HashMap;

use serde::Deserialize;

use super::Value;

/// Model a single store manifest as part of the [`StoresManifest`] catalogue.
#[derive(Debug, Deserialize)]
pub struct StoreManifest {
    /// Location of the [`VersionsManifest`] file for this store rule.
    pub manifest: String,

    /// Values that must match the attributes from the lookup request to select this store.
    #[serde(default)]
    pub matchers: HashMap<String, Value>,

    /// ID of the store that must match the lookup request to select this store.
    pub store: String,
}

/// Model the on-disk manifest catalogue of known stores.
#[derive(Debug, Deserialize)]
pub struct StoresManifest {
    /// List of [`StoreManifest`]s defined in the catalogue file.
    pub stores: Vec<StoreManifest>,
}

/// Model a single store version manifest as part of the [`StoresManifest`] catalogue.
#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    /// Values that must match the attributes from the lookup request to select this version.
    #[serde(default)]
    pub matchers: HashMap<String, Value>,

    /// Options to load templates selected by this rule.
    pub template: VersionTemplate,

    /// Semantic version requirements to select this version.
    pub version: String,
}

/// Model a store's version template information as part of the [`StoresManifest`] catalogue.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VersionTemplate {
    /// The case where the template is specified as an object with attached options.
    Options {
        /// Location of the templates for this store's version.
        target: String,

        /// Additional options to load the template with.
        #[serde(default)]
        options: serde_json::Value,
    },

    /// Location of the templates for this store's version.
    Simple(String),
}

impl From<VersionTemplate> for super::TemplateLoadOptions {
    fn from(value: VersionTemplate) -> Self {
        match value {
            VersionTemplate::Options { target, options } => Self {
                template: target,
                options,
            },
            VersionTemplate::Simple(target) => Self {
                template: target,
                options: Default::default(),
            },
        }
    }
}

/// Model the on-disk catalogue of versions available for a specific store manifest.
#[derive(Debug, Deserialize)]
pub struct VersionsManifest {
    /// List of [`VersionManifest`]s defined in the catalogue file.
    pub versions: Vec<VersionManifest>,
}
