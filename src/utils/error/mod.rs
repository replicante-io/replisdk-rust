//! Utilities to deal with errors.

#[cfg(feature = "utils-error_slog")]
pub mod slog;

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
        let error_trail = error_trail.join("\n  ");
        document.insert("error_trail".into(), error_trail.into());
    }

    // Attach a backtrace if available.
    let backtrace = error.backtrace().to_string();
    if !backtrace.is_empty() && backtrace != crate::utils::BACKTRACE_DISABLED {
        document.insert("error_backtrace".into(), backtrace.into());
    }

    serde_json::Value::Object(document)
}
