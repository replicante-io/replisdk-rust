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

/// Reference to a typed value of a `Node` attribute.
#[derive(Clone, Debug, Default, Eq)]
pub enum AttributeValueRef<'a> {
    /// Represents a boolean attribute value.
    Boolean(bool),

    /// Represents an attribute without a value.
    #[default]
    Null,

    /// Represents a numeric attribute, based on owned JSON number representation.
    Number(Number),

    /// Represents a numeric attribute, based on borrowed JSON number representation.
    NumberRef(&'a Number),

    /// Represents a string attribute.
    String(&'a str),
}

impl<'a> std::cmp::PartialEq<AttributeValueRef<'a>> for AttributeValueRef<'a> {
    fn eq(&self, other: &AttributeValueRef<'a>) -> bool {
        match (self, other) {
            (AttributeValueRef::Boolean(me), AttributeValueRef::Boolean(other)) => me.eq(other),
            (AttributeValueRef::Null, AttributeValueRef::Null) => true,
            (AttributeValueRef::Number(me), AttributeValueRef::Number(other)) => me.eq(other),
            (AttributeValueRef::Number(me), AttributeValueRef::NumberRef(other)) => me.eq(other),
            (AttributeValueRef::NumberRef(me), AttributeValueRef::Number(other)) => me.eq(&other),
            (AttributeValueRef::NumberRef(me), AttributeValueRef::NumberRef(other)) => me.eq(other),
            (AttributeValueRef::String(me), AttributeValueRef::String(other)) => me.eq(other),
            _ => false,
        }
    }
}

impl<'a> From<&'a AttributeValue> for AttributeValueRef<'a> {
    fn from(value: &'a AttributeValue) -> Self {
        match value {
            AttributeValue::Boolean(value) => AttributeValueRef::Boolean(*value),
            AttributeValue::Null => AttributeValueRef::Null,
            AttributeValue::Number(value) => AttributeValueRef::NumberRef(value),
            AttributeValue::String(value) => AttributeValueRef::String(value),
        }
    }
}

impl From<u64> for AttributeValueRef<'_> {
    fn from(value: u64) -> Self {
        let number = Number::from(value);
        AttributeValueRef::Number(number)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Number;

    use super::AttributeValueRef;

    #[rstest::rstest]
    #[case(
        AttributeValueRef::Boolean(true),
        AttributeValueRef::Boolean(true),
        true
    )]
    #[case(
        AttributeValueRef::Boolean(true),
        AttributeValueRef::Boolean(false),
        false
    )]
    #[case(AttributeValueRef::Null, AttributeValueRef::Null, true)]
    #[case(
        AttributeValueRef::Number(Number::from(1)),
        AttributeValueRef::Number(Number::from(1)),
        true
    )]
    #[case(
        AttributeValueRef::Number(Number::from(1)),
        AttributeValueRef::Number(Number::from(2)),
        false
    )]
    #[case(
        AttributeValueRef::String("same"),
        AttributeValueRef::String("same"),
        true
    )]
    #[case(
        AttributeValueRef::String("left"),
        AttributeValueRef::String("right"),
        false
    )]
    #[case(AttributeValueRef::Null, AttributeValueRef::Boolean(true), false)]
    #[case(
        AttributeValueRef::Null,
        AttributeValueRef::Number(Number::from(0)),
        false
    )]
    #[case(AttributeValueRef::Null, AttributeValueRef::String("some"), false)]
    fn attribute_value_ref_eq(
        #[case] left: AttributeValueRef<'_>,
        #[case] right: AttributeValueRef<'_>,
        #[case] should_eq: bool,
    ) {
        let are_eq = left.eq(&right);
        assert_eq!(are_eq, should_eq);
    }

    #[rstest::rstest]
    #[case(1, 1, true)]
    #[case(1, 2, false)]
    fn attribute_value_ref_eq_number_ref(
        #[case] left: u64,
        #[case] right: u64,
        #[case] should_eq: bool,
    ) {
        let left = Number::from(left);
        let left = AttributeValueRef::NumberRef(&left);
        let right = Number::from(right);
        let right = AttributeValueRef::NumberRef(&right);
        let are_eq = left.eq(&right);
        assert_eq!(are_eq, should_eq);
    }

    #[rstest::rstest]
    #[case(1, 1, true)]
    #[case(1, 2, false)]
    fn attribute_value_ref_eq_number_ref_left(
        #[case] left: u64,
        #[case] right: u64,
        #[case] should_eq: bool,
    ) {
        let left = AttributeValueRef::Number(Number::from(left));
        let right = Number::from(right);
        let right = AttributeValueRef::NumberRef(&right);
        let are_eq = left.eq(&right);
        assert_eq!(are_eq, should_eq);
    }

    #[rstest::rstest]
    #[case(1, 1, true)]
    #[case(1, 2, false)]
    fn attribute_value_ref_eq_number_ref_right(
        #[case] left: u64,
        #[case] right: u64,
        #[case] should_eq: bool,
    ) {
        let left = Number::from(left);
        let left = AttributeValueRef::NumberRef(&left);
        let right = AttributeValueRef::Number(Number::from(right));
        let are_eq = left.eq(&right);
        assert_eq!(are_eq, should_eq);
    }
}
