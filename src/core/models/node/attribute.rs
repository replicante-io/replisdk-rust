//! Models to describe node attributes and matching operations.
use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;

use crate::agent::models::AttributeValue;

/// Operation and values to match against node's [`AttributeValue`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeMatcher {
    /// Match using a complex operation.
    Complex(AttributeMatcherComplex),

    /// Match nodes when the attribute equals the given value.
    Eq(AttributeValue),

    /// Match nodes when the attribute is in the given list of values.
    In(Vec<AttributeValue>),
}

/// Match using a complex operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributeMatcherComplex {
    /// The matching operation to apply.
    pub op: AttributeMatcherOp,

    /// The expected value to match against, for single value operations.
    #[serde(default)]
    pub value: Option<AttributeValue>,

    /// The list of expected values to match against, for multi-value operations.
    #[serde(default)]
    pub values: Option<Vec<AttributeValue>>,
}

/// The matching operation to apply.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AttributeMatcherOp {
    /// Match when the attribute value is equal to the expected value.
    #[serde(alias = "eq")]
    Eq,

    /// Match when the attribute value is in the expected values list.
    #[serde(alias = "in")]
    In,

    /// Match when the attribute value is not equal to the expected value.
    #[serde(alias = "ne")]
    Ne,

    /// Match when the attribute value is not in the expected values list.
    #[serde(alias = "not-in", alias = "not_in")]
    NotIn,

    /// Match when the attribute value is not null.
    #[serde(alias = "set")]
    Set,

    /// Match when the attribute value is null.
    #[serde(alias = "unset")]
    Unset,
}

impl std::fmt::Display for AttributeMatcherOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "Eq"),
            Self::In => write!(f, "In"),
            Self::Ne => write!(f, "Ne"),
            Self::NotIn => write!(f, "NotIn"),
            Self::Set => write!(f, "Set"),
            Self::Unset => write!(f, "Unset"),
        }
    }
}

/// Reference to a typed value of a [`Node`] attribute.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum AttributeValueRef<'a> {
    /// Represents a boolean attribute value.
    Boolean(bool),

    /// Represents an attribute without a value.
    #[default]
    Null,

    /// Represents a numeric attribute, based on JSON number representation.
    Number(&'a Number),

    /// Represents a string attribute.
    String(&'a str),
}

impl<'a> From<&'a AttributeValue> for AttributeValueRef<'a> {
    fn from(value: &'a AttributeValue) -> Self {
        match value {
            AttributeValue::Boolean(value) => AttributeValueRef::Boolean(*value),
            AttributeValue::Null => AttributeValueRef::Null,
            AttributeValue::Number(value) => AttributeValueRef::Number(value),
            AttributeValue::String(value) => AttributeValueRef::String(value),
        }
    }
}
