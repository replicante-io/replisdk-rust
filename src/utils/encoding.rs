//! Utilities to encode and decode advanced types into storable data.
use anyhow::Context;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use time::OffsetDateTime;

/// Convert nanoseconds from/to u32 to the fractal portion of f64.
const NANO_SEC_UNIT: f64 = 1_000_000_000.0;

/// Errors when encoding or decoding data.
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

/// Decode an [`OffsetDateTime`](time::OffsetDateTime) from an f64.
///
/// The f64 value encodes:
///
/// - The unix timestamp in seconds as the integer part.
/// - The nanosecond portion of the time as the fractal part.
pub fn decode_time_f64(value: f64) -> Result<OffsetDateTime> {
    let unix = value.floor() as i64;
    let nanos = (value.fract() * NANO_SEC_UNIT) as u32;
    let time = OffsetDateTime::from_unix_timestamp(unix)?;
    let time = time.replace_nanosecond(nanos)?;
    Ok(time)
}

/// Decode an optional [`OffsetDateTime`](time::OffsetDateTime) from an f64.
///
/// The encoded f64 follows the format described in [`decode_time_f64`].
pub fn decode_time_option_f64(value: Option<f64>) -> Result<Option<OffsetDateTime>> {
    let value = match value {
        None => return Ok(None),
        Some(value) => value,
    };
    decode_time_f64(value).map(Some)
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

/// Encode an [`OffsetDateTime`](time::OffsetDateTime) into an f64.
///
/// The encoded f64 follows the format described in [`decode_time_f64`].
pub fn encode_time_f64(value: OffsetDateTime) -> Result<f64> {
    let unix = value.unix_timestamp() as f64;
    let nanos = f64::from(value.nanosecond());
    Ok(unix + (nanos / NANO_SEC_UNIT))
}

/// Encode an optional [`OffsetDateTime`](time::OffsetDateTime) into an f64.
///
/// The encoded f64 follows the format described in [`decode_time_f64`].
pub fn encode_time_option_f64(value: Option<OffsetDateTime>) -> Result<Option<f64>> {
    let value = match value {
        None => return Ok(None),
        Some(value) => value,
    };
    encode_time_f64(value).map(Some)
}

#[cfg(test)]
mod tests {
    #[test]
    fn decode_time_f64() {
        let time = 1680670808.12345;
        let time = super::decode_time_f64(time).unwrap();
        let expected = time::OffsetDateTime::parse(
            "2023-04-05T05:00:08.12345004Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        assert_eq!(expected, time);
    }

    #[test]
    fn encode_time_f64() {
        let time = time::OffsetDateTime::parse(
            "2023-04-05T05:00:08.12345Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let time = super::encode_time_f64(time).unwrap();
        assert_eq!(time, 1680670808.12345);
    }

    #[test]
    fn ensure_idempotency() {
        let expected = 1680670808.12345;
        let time = super::decode_time_f64(expected).unwrap();
        let time = super::encode_time_f64(time).unwrap();
        assert_eq!(expected, time);
    }
}
