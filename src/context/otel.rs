//! Additional [`Context`] feature to integrate with Open Telemetry.
// Support OpenTelemetry both via the API layer or the full SDK.
#[cfg(all(feature = "opentelemetry", not(feature = "opentelemetry_api")))]
use {
    opentelemetry::trace::TraceContextExt, opentelemetry::trace::TraceId,
    opentelemetry::Context as OtelContext,
};
#[cfg(feature = "opentelemetry_api")]
use {
    opentelemetry_api::trace::TraceContextExt, opentelemetry_api::trace::TraceId,
    opentelemetry_api::Context as OtelContext,
};

use super::ContextBuilder;

impl ContextBuilder {
    /// Decorate the [`Context`]'s logger with the trace ID of the current OpenTelemetry span.
    ///
    /// [`Context`]: super::Context
    pub fn log_trace(self) -> Self {
        let context = OtelContext::current();
        let span = context.span();
        let trace_id = span.span_context().trace_id();
        if trace_id == TraceId::INVALID {
            self
        } else {
            let trace_id = trace_id.to_string();
            self.log_values(slog::o!("trace_id" => trace_id))
        }
    }
}
