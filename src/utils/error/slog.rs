//! Standard error formatting as [`slog`] key/value pairs.
use anyhow::Error;
use slog::Record;
use slog::Serializer;
use slog::KV;

/// Borrow an [`Error`] to attach structured information to [`slog`] events.
pub struct ErrorAttributes<'a> {
    error: &'a Error,
}

impl KV for ErrorAttributes<'_> {
    fn serialize(&self, _: &Record, serializer: &mut dyn Serializer) -> slog::Result {
        // Essential error information.
        let error_cause = self.error.root_cause().to_string();
        let error_msg = self.error.to_string();
        serializer.emit_str("error_msg", &error_msg)?;
        if error_msg != error_cause {
            serializer.emit_str("error_cause", &error_cause)?;
        }

        // Emit the full error trail where intermediate messages are present.
        let error_trail: Vec<String> = self.error.chain().map(ToString::to_string).collect();
        if error_trail.len() > 2 {
            let error_trail = error_trail.join("\n  ");
            serializer.emit_str("error_trail", &error_trail)?;
        }

        // Attach a backtrace if available.
        let backtrace = self.error.backtrace().to_string();
        if !backtrace.is_empty() && backtrace != crate::utils::BACKTRACE_DISABLED {
            serializer.emit_str("error_backtrace", &backtrace)?;
        }
        Ok(())
    }
}

impl<'a> From<&'a Error> for ErrorAttributes<'a> {
    fn from(error: &'a Error) -> Self {
        ErrorAttributes { error }
    }
}
