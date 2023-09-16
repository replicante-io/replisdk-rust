//! Helpers for Agent SDK tracing.
use opentelemetry_api::global::BoxedTracer;
use opentelemetry_api::trace::SpanKind;
use opentelemetry_api::trace::TraceContextExt;
use opentelemetry_api::trace::Tracer;
use opentelemetry_api::trace::TracerProvider;
use opentelemetry_api::Context;

/// Short-hand to create a tracer for the Agent SDK library.
pub fn tracer() -> BoxedTracer {
    opentelemetry_api::global::tracer_provider().versioned_tracer(
        env!("CARGO_PKG_NAME"),
        Some(env!("CARGO_PKG_VERSION")),
        Option::<&str>::None,
        None,
    )
}

/// Initialised a new span and context for Agent Store operations,
///
/// The new span and context are automatically children of the active span and context.
pub fn store_op_context(op: &str) -> Context {
    let op = format!("store.{}", op);
    let tracer = self::tracer();
    let mut builder = tracer.span_builder(op);
    builder.span_kind = Some(SpanKind::Client);
    let parent = Context::current();
    let span = tracer.build_with_context(builder, &parent);
    parent.with_span(span)
}
