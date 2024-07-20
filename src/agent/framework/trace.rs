//! Helpers for Agent SDK tracing.
use opentelemetry::global::BoxedTracer;
use opentelemetry::trace::SpanKind;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::trace::Tracer;
use opentelemetry::trace::TracerProvider;
use opentelemetry::Context;

/// Short-hand to create a tracer for the Agent SDK library.
pub fn tracer() -> BoxedTracer {
    opentelemetry::global::tracer_provider()
        .tracer_builder(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .build()
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
