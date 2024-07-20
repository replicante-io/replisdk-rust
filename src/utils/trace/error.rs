//! Decorate [`Result`]s and [`Future`]s to trace occurred errors.
use std::future::Future;
use std::pin::Pin;
use std::result::Result;
use std::task::Context as TaskContext;
use std::task::Poll;

use anyhow::Error;
use opentelemetry::trace::Status;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::Context;
use opentelemetry::KeyValue;

// --- Trait definitions for sync errors --- //
/// Extend [`Result`]s with [`anyhow::Error`]s to trace occurred errors.
pub trait TraceErrExt {
    /// For `Err`s, record an error event against the current OpenTelemetry context.
    fn trace_on_err(self) -> Self;

    /// For `Err`s, record an error event and mark the current OpenTelemetry context as failed.
    fn trace_on_err_with_status(self) -> Self;
}

/// Extend [`Result`]s with [`std::error::Error`]s to trace occurred errors.
pub trait TraceStdErrExt {
    /// For `Err`s, record an error event against the current OpenTelemetry context.
    fn trace_on_err(self) -> Self;

    /// For `Err`s, record an error event and mark the current OpenTelemetry context as failed.
    fn trace_on_err_with_status(self) -> Self;
}

// --- Trait definitions for async errors --- //
/// Extend [`Future`]s that return [`Result`]s with [`anyhow::Error`]s to trace occurred errors.
pub trait TraceFutureErrExt<T>
where
    Self: Future<Output = Result<T, Error>>,
    Self: Sized,
{
    /// For `Err`s, record an error event against the current OpenTelemetry context.
    fn trace_on_err(self) -> WithTraceFutureErr<Self, T>;

    /// For `Err`s, record an error event and mark the current OpenTelemetry context as failed.
    fn trace_on_err_with_status(self) -> WithTraceFutureErr<Self, T>;
}

/// Extend [`Future`]s that return [`Result`]s with [`std::error::Error`]s to trace occurred errors.
pub trait TraceFutureStdErrExt<T, E>
where
    Self: Future<Output = Result<T, E>>,
    Self: Sized,
    E: std::error::Error,
{
    /// For `Err`s, record an error event against the current OpenTelemetry context.
    fn trace_on_err(self) -> WithTraceFutureStdErr<E, Self, T>;

    /// For `Err`s, record an error event and mark the current OpenTelemetry context as failed.
    fn trace_on_err_with_status(self) -> WithTraceFutureStdErr<E, Self, T>;
}

// --- Macro to streamline trait impl --- //
/// Shortcut to only work on `Err` variants.
macro_rules! impl_on_error {
    ($result:expr) => {
        match $result {
            Err(error) => error,
            Ok(value) => return Ok(value),
        }
    };
}

/// Implement `Future::poll` for `WithTraceFuture*Err` types.
macro_rules! impl_poll {
    ($self:expr, $cx:expr) => {{
        let this = $self.project();
        match this.inner.poll($cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(value) => {
                let value = if *this.with_status {
                    value.trace_on_err_with_status()
                } else {
                    value.trace_on_err()
                };
                Poll::Ready(value)
            }
        }
    }};
}

/// Reusable block to record anyhow errors onto a span.
macro_rules! impl_record_anyhow {
    ($span:expr, $error:expr) => {{
        if $span.is_recording() {
            let attributes = vec![KeyValue::new("exception.message", $error.to_string())];
            $span.add_event("exception", attributes);
        }
    }};
}

// --- Trait Implementations for sync errors --- //
impl<T> TraceErrExt for Result<T, Error> {
    fn trace_on_err(self) -> Self {
        // Ignore successes.
        let error = impl_on_error!(self);

        // Trace the error event and return the original error.
        let context = Context::current();
        let span = context.span();
        impl_record_anyhow!(span, error);
        Err(error)
    }

    fn trace_on_err_with_status(self) -> Self {
        // Ignore successes.
        let error = impl_on_error!(self);

        // Trace the error event and return the original error.
        let context = Context::current();
        let span = context.span();
        impl_record_anyhow!(span, error);
        span.set_status(Status::error(error.to_string()));
        Err(error)
    }
}

impl<T, E> TraceStdErrExt for Result<T, E>
where
    E: std::error::Error,
{
    fn trace_on_err(self) -> Self {
        // Ignore successes.
        let error = impl_on_error!(self);

        // Trace the error event and return the original error.
        let context = Context::current();
        let span = context.span();
        span.record_error(&error);
        Err(error)
    }

    fn trace_on_err_with_status(self) -> Self {
        // Ignore successes.
        let error = impl_on_error!(self);

        // Trace the error event and return the original error.
        let context = Context::current();
        let span = context.span();
        span.record_error(&error);
        span.set_status(Status::error(error.to_string()));
        Err(error)
    }
}

// --- Trait Implementations for async errors --- //
impl<F, T> TraceFutureErrExt<T> for F
where
    F: Future<Output = Result<T, Error>>,
{
    fn trace_on_err(self) -> WithTraceFutureErr<F, T> {
        WithTraceFutureErr {
            inner: self,
            with_status: false,
        }
    }

    fn trace_on_err_with_status(self) -> WithTraceFutureErr<F, T> {
        WithTraceFutureErr {
            inner: self,
            with_status: true,
        }
    }
}

impl<E, F, T> TraceFutureStdErrExt<T, E> for F
where
    E: std::error::Error,
    F: Future<Output = Result<T, E>>,
{
    fn trace_on_err(self) -> WithTraceFutureStdErr<E, Self, T> {
        WithTraceFutureStdErr {
            inner: self,
            with_status: false,
        }
    }

    fn trace_on_err_with_status(self) -> WithTraceFutureStdErr<E, Self, T> {
        WithTraceFutureStdErr {
            inner: self,
            with_status: true,
        }
    }
}

// --- Future type for async traits --- //
pin_project_lite::pin_project! {
    /// Wrap a fallible future to trace errors when it completes.
    pub struct WithTraceFutureErr<F, T>
    where
        F: Future<Output = Result<T, Error>>,
    {
        #[pin]
        inner: F,
        with_status: bool,
    }
}

pin_project_lite::pin_project! {
    /// Wrap a fallible future to trace errors when it completes.
    pub struct WithTraceFutureStdErr<E, F, T>
    where
        E: std::error::Error,
        F: Future<Output = Result<T, E>>,
    {
        #[pin]
        inner: F,
        with_status: bool,
    }
}

impl<F, T> Future for WithTraceFutureErr<F, T>
where
    F: Future<Output = Result<T, Error>>,
{
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        impl_poll!(self, cx)
    }
}

impl<E, F, T> Future for WithTraceFutureStdErr<E, F, T>
where
    E: std::error::Error,
    F: Future<Output = Result<T, E>>,
{
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        impl_poll!(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::trace::FutureExt;
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry::trace::Tracer;

    use super::TraceErrExt;
    use super::TraceFutureErrExt;
    use super::TraceFutureStdErrExt;
    use super::TraceStdErrExt;

    #[derive(Debug, thiserror::Error)]
    #[error("test")]
    pub struct TestStdError;

    #[test]
    fn trace_error() {
        let tracer = opentelemetry::global::tracer("test");
        let span = tracer.start("test");
        let context = opentelemetry::Context::current();
        let context = context.with_span(span);
        let _guard = context.clone().attach();

        let error: anyhow::Result<()> = Err(anyhow::anyhow!("test"));
        let _ = error.trace_on_err();
    }

    #[tokio::test]
    async fn trace_future_error() {
        let tracer = opentelemetry::global::tracer("test");
        let span = tracer.start("test");
        let context = opentelemetry::Context::current();
        let context = context.with_span(span);

        let error = async {
            let error: anyhow::Result<()> = Err(anyhow::anyhow!("test"));
            error
        };
        let _ = error.trace_on_err().with_context(context).await;
    }

    #[tokio::test]
    async fn trace_future_std_error() {
        let tracer = opentelemetry::global::tracer("test");
        let span = tracer.start("test");
        let context = opentelemetry::Context::current();
        let context = context.with_span(span);

        let error = async {
            let error: std::result::Result<(), TestStdError> = Err(TestStdError);
            error
        };
        let _ = error.trace_on_err().with_context(context).await;
    }

    #[test]
    fn trace_std_error() {
        let tracer = opentelemetry::global::tracer("test");
        let span = tracer.start("test");
        let context = opentelemetry::Context::current();
        let context = context.with_span(span);
        let _guard = context.clone().attach();

        let error: std::result::Result<(), TestStdError> = Err(TestStdError);
        let _ = error.trace_on_err();
    }
}
