//! Models to describe node attributes.
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;

/// Typed value of a Node attribute.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    /// Represents a boolean attribute value.
    Boolean(bool),

    /// Represents an attribute without a value.
    #[default]
    Null,

    /// Represents a numeric attribute, based on JSON number representation.
    Number(Number),

    /// Represents a string attribute.
    String(String),
}

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        AttributeValue::Boolean(value)
    }
}

impl From<Number> for AttributeValue {
    fn from(value: Number) -> Self {
        AttributeValue::Number(value)
    }
}

impl<'a> From<&'a str> for AttributeValue {
    fn from(value: &'a str) -> Self {
        AttributeValue::String(value.to_string())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        AttributeValue::String(value)
    }
}

/// Map of Node attribute identifies to values.
pub type AttributesMap = BTreeMap<String, AttributeValue>;
