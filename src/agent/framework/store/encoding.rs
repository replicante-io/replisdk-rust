//! Functions to encode and decode advanced types into storable data.
use anyhow::Context;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    /// Unable to encode structured data as a JSON string.
    #[error("unable to encode structured data as a JSON string")]
    AsJson,

    /// Unable to decode structured data from a JSON string.
    #[error("unable to decode structured data from a JSON string")]
    FromJson,

    /// Unable to encode time as a string.
    #[error("unable to encode time as a string")]
    TimeEncode,

    /// Unable to decode time from a string.
    #[error("unable to decode time from a string")]
    TimeDecode,
}

/// Decode a [`serde`] deserializable type from a string.
pub fn decode_serde<V>(value: &str) -> Result<V>
where
    V: DeserializeOwned,
{
    serde_json::from_str(value)
        .context(EncodeError::FromJson)
        .map_err(anyhow::Error::from)
}

/// Decode an optional [`serde`] deserializable type from a string.
pub fn decode_serde_option<V>(value: &Option<String>) -> Result<Option<V>>
where
    V: DeserializeOwned,
{
    let value = match value {
        None => return Ok(None),
        Some(value) => value,
    };
    decode_serde(value).map(Some)
}

/// Decode an [`OffsetDateTime`](time::OffsetDateTime) from an RFC3339 string.
pub fn decode_time(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .context(EncodeError::TimeDecode)
        .map_err(anyhow::Error::from)
}

/// Decode an [`OffsetDateTime`](time::OffsetDateTime) from an RFC3339 string.
pub fn decode_time_option(value: &Option<String>) -> Result<Option<OffsetDateTime>> {
    let value = match value {
        None => return Ok(None),
        Some(value) => value,
    };
    decode_time(value).map(Some)
}

/// Encode a [`serde`] serialisable type into a string.
pub fn encode_serde<V>(value: &V) -> Result<String>
where
    V: Serialize,
{
    serde_json::to_string(value)
        .context(EncodeError::AsJson)
        .map_err(anyhow::Error::from)
}

/// Encode an optional [`serde`] serialisable type into a string.
pub fn encode_serde_option<V>(value: &Option<V>) -> Result<Option<String>>
where
    V: Serialize,
{
    let value = match value {
        None => return Ok(None),
        Some(value) => value,
    };
    encode_serde(value).map(Some)
}

/// Encode an [`OffsetDateTime`](time::OffsetDateTime) into an RFC3339 string.
pub fn encode_time(value: OffsetDateTime) -> Result<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .context(EncodeError::TimeEncode)
        .map_err(anyhow::Error::from)
}

/// Encode an optional [`OffsetDateTime`](time::OffsetDateTime) into an RFC3339 string.
pub fn encode_time_option(value: Option<OffsetDateTime>) -> Result<Option<String>> {
    let value = match value {
        None => return Ok(None),
        Some(value) => value,
    };
    encode_time(value).map(Some)
}
