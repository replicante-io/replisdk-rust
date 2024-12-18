//! Helpers for Agent SDK tracing.
use once_cell::sync::Lazy;
use opentelemetry::global::BoxedTracer;
use opentelemetry::trace::SpanKind;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::trace::Tracer;
use opentelemetry::Context;

/// Tracer for the Agent SDK library.
pub static TRACER: Lazy<BoxedTracer> = Lazy::new(|| {
    let scope = opentelemetry::InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .build();
    opentelemetry::global::tracer_with_scope(scope)
});

/// Initialised a new span and context for Agent Store operations,
///
/// The new span and context are automatically children of the active span and context.
pub fn store_op_context(op: &str) -> Context {
    let op = format!("store.{}", op);
    let mut builder = TRACER.span_builder(op);
    builder.span_kind = Some(SpanKind::Client);
    let parent = Context::current();
    let span = TRACER.build_with_context(builder, &parent);
    parent.with_span(span)
}
