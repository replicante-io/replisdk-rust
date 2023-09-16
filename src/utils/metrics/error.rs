//! Decorate [`Result`]s and [`Future`]s to count occurred errors.
use std::future::Future;
use std::pin::Pin;
use std::task::Context as TaskContext;
use std::task::Poll;

use prometheus::Counter;

/// Extend [`Result`]s to count occurred errors.
pub trait CountErrExt {
    /// For `Err`s, increment the counter by 1.
    fn count_on_err(self, counter: Counter) -> Self;
}

impl<E, T> CountErrExt for Result<T, E> {
    fn count_on_err(self, counter: Counter) -> Self {
        // Ignore successes.
        let error = match self {
            Err(error) => error,
            Ok(value) => return Ok(value),
        };

        counter.inc();
        Err(error)
    }
}

/// Extend [`Future`]s that return [`Result`]s to count occurred errors.
pub trait CountFutureErrExt<E, T>
where
    Self: Future<Output = Result<T, E>>,
    Self: Sized,
{
    /// For `Err`s, increment the counter by 1.
    fn count_on_err(self, counter: Counter) -> WithCountFutureErr<E, Self, T>;
}

impl<E, F, T> CountFutureErrExt<E, T> for F
where
    F: Future<Output = Result<T, E>>,
{
    fn count_on_err(self, counter: Counter) -> WithCountFutureErr<E, Self, T> {
        WithCountFutureErr {
            counter,
            inner: self,
        }
    }
}

pin_project_lite::pin_project! {
    /// Wrap a fallible future to trace errors when it completes.
    pub struct WithCountFutureErr<E, F, T>
    where
        F: Future<Output = Result<T, E>>,
    {
        counter: Counter,
        #[pin]
        inner: F,
    }
}

impl<E, F, T> Future for WithCountFutureErr<E, F, T>
where
    F: Future<Output = Result<T, E>>,
{
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(value) => Poll::Ready(value.count_on_err(this.counter.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use prometheus::CounterVec;
    use prometheus::Opts;

    use super::CountErrExt;
    use super::CountFutureErrExt;

    #[test]
    fn count_error() {
        let counter = CounterVec::new(Opts::new("test", "test vector"), &["op"]).unwrap();
        let instance = counter.with_label_values(&["test"]);

        let error: anyhow::Result<()> = Err(anyhow::anyhow!("test"));
        let _ = error.count_on_err(instance.clone());

        let current = instance.get();
        assert_eq!(current, 1.0);
    }

    #[tokio::test]
    async fn count_future_error() {
        let counter = CounterVec::new(Opts::new("test", "test vector"), &["op"]).unwrap();
        let instance = counter.with_label_values(&["test"]);

        let error = async {
            let error: anyhow::Result<()> = Err(anyhow::anyhow!("test"));
            error
        };
        let _ = error.count_on_err(instance.clone()).await;

        let current = instance.get();
        assert_eq!(current, 1.0);
    }
}
