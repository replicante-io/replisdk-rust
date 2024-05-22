//! Utilities to deal with errors.
#[cfg(feature = "utils-error_json")]
use anyhow::Context;

#[cfg(feature = "utils-error_slog")]
pub mod slog;

/// Wrap a remote error message decoded from JSON.
#[cfg(feature = "utils-error_json")]
#[derive(Debug, thiserror::Error)]
#[error("[remote] {message}")]
pub struct RemoteError {
    /// Original error message reported by a remote system.
    pub message: String,
}

#[cfg(feature = "utils-error_json")]
impl From<&str> for RemoteError {
    fn from(value: &str) -> Self {
        let message = value.to_string();
        RemoteError { message }
    }
}

#[cfg(feature = "utils-error_json")]
impl From<String> for RemoteError {
    fn from(value: String) -> Self {
        let message = value;
        RemoteError { message }
    }
}

/// Unable to decode a remote error.
#[cfg(feature = "utils-error_json")]
#[derive(Debug, thiserror::Error)]
#[error("unable to decode remote error")]
pub struct DecodeError;

/// Utility function to decode an error from a JSON object.
///
/// This function is the inverse of [`into_json`].
#[cfg(feature = "utils-error_json")]
pub fn from_json(payload: serde_json::Value) -> anyhow::Result<anyhow::Error> {
    // Check the payload has the error flag we expect from `into_json` conversion.
    match payload.get("error") {
        None => {
            let error = anyhow::anyhow!("remote error must have the error flag");
            return Err(error.context(DecodeError));
        }
        Some(error) if error.is_boolean() && error.as_bool().unwrap_or(false) => (),
        Some(_) => {
            let error = anyhow::anyhow!("remote error must have the error flag set to true");
            return Err(error.context(DecodeError));
        }
    }

    // If the error has a trail use that for everything.
    if let Some(trail) = payload.get("error_trail") {
        let trail: Vec<String> = serde_json::from_value(trail.clone()).context(DecodeError)?;
        let mut trail: Vec<_> = trail
            .into_iter()
            .map(|error| anyhow::anyhow!(RemoteError::from(error)))
            .collect();
        trail.reverse();
        let mut trail = trail.into_iter();
        let mut error = trail.next().ok_or_else(|| {
            anyhow::anyhow!("error_trail must contain at least one error").context(DecodeError)
        })?;
        for layer in trail {
            error = error.context(layer);
        }
        return Ok(error);
    }

    // Create an error from the error message.
    let msg = payload
        .get("error_msg")
        .ok_or_else(|| anyhow::anyhow!("remote error must have an error_msg").context(DecodeError))?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("error_msg must be a string").context(DecodeError))?;
    let msg = anyhow::anyhow!(RemoteError::from(msg));

    // If present, track the error cause too.
    let cause = match payload.get("error_cause") {
        None => None,
        Some(cause) => {
            let cause = cause.as_str().ok_or_else(|| {
                anyhow::anyhow!("error_cause must be a string").context(DecodeError)
            })?;
            Some(anyhow::anyhow!(RemoteError::from(cause)))
        }
    };

    // Return the final error.
    let error = match cause {
        None => msg,
        Some(cause) => cause.context(msg),
    };
    Ok(error)
}

/// Utility function to encode an error into a JSON object.
#[cfg(feature = "utils-error_json")]
pub fn into_json(error: anyhow::Error) -> serde_json::Value {
    let mut document = serde_json::Map::default();

    let error_cause = error.root_cause().to_string();
    let error_msg = error.to_string();
    if error_msg != error_cause {
        document.insert("error_cause".into(), error_cause.into());
    }
    document.insert("error_msg".into(), error_msg.into());

    // Emit the full error trail where intermediate messages are present.
    let error_trail: Vec<String> = error.chain().map(ToString::to_string).collect();
    if error_trail.len() > 2 {
        document.insert("error_trail".into(), error_trail.into());
    }

    // Attach a backtrace if available.
    let backtrace = error.backtrace().to_string();
    if !backtrace.is_empty() && backtrace != crate::utils::BACKTRACE_DISABLED {
        document.insert("error_backtrace".into(), backtrace.into());
    }

    serde_json::Value::Object(document)
}

#[cfg(all(test, feature = "utils-error_json"))]
mod tests {
    #[test]
    fn decode_message() {
        let payload = serde_json::json!({
            "error": true,
            "error_msg": "msg",
        });
        let error = super::from_json(payload).unwrap();
        let error = format!("{:?}", error);
        assert_eq!(error, "[remote] msg");
    }

    #[test]
    fn decode_cause() {
        let payload = serde_json::json!({
            "error": true,
            "error_msg": "msg",
            "error_cause": "cause",
        });
        let error = super::from_json(payload).unwrap();
        let error = format!("{:?}", error);
        assert_eq!(
            error,
            r#"[remote] msg

Caused by:
    [remote] cause"#
        );
    }

    #[test]
    fn decode_trail() {
        let payload = serde_json::json!({
            "error": true,
            "error_msg": "msg",
            "error_cause": "cause",
            "error_trail": [
                "trail msg",
                "trail step 1",
                "trail step 2",
                "trail cause",
            ]
        });
        let error = super::from_json(payload).unwrap();
        let error = format!("{:?}", error);
        assert_eq!(
            error,
            r#"[remote] trail msg

Caused by:
    0: [remote] trail step 1
    1: [remote] trail step 2
    2: [remote] trail cause"#
        );
    }
}
