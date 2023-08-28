//! Utilities to introspect applications and libraries with traces more easley.
use std::borrow::Cow;

use opentelemetry_api::trace::TraceContextExt;
use opentelemetry_api::Context;

mod error;

pub use self::error::TraceErrExt;
pub use self::error::TraceFutureErrExt;
pub use self::error::TraceFutureStdErrExt;
pub use self::error::TraceStdErrExt;

/// Create a root span and context.
pub fn root<N, T>(tracer: &T, name: N) -> Context
where
    N: Into<Cow<'static, str>>,
    T: opentelemetry_api::trace::Tracer,
    T::Span: Send + Sync + 'static,
{
    let empty = Context::new();
    let root = tracer.start_with_context(name, &empty);
    empty.with_span(root)
}
