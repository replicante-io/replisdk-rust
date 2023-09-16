use std::collections::HashMap;
use std::future::ready;
use std::future::Ready;
use std::time::Instant;

use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use prometheus::core::Collector;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;

const DEFAULT_METRIC_DURATIONS_DESC: &str = "Duration of handled requests";
const DEFAULT_METRIC_ERRORS_DESC: &str = "Number of requests failed with unhandled errors";

/// An [`actix_web`] middleware to collect request metrics.
///
/// Collected metrics are:
///
/// - Histogram of request durations, by method, path and response status.
/// - Number of requests that failed with unhandled errors, by method and path.
#[derive(Clone)]
pub struct MetricsCollector {
    durations: HistogramVec,
    errors: CounterVec,
}

impl MetricsCollector {
    /// Build a new `MetricsCollector` middleware.
    pub fn build() -> MetricsCollectorBuilder {
        MetricsCollectorBuilder::default()
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsCollector
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsCollectorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let collector = self.clone();
        let middleware = MetricsCollectorMiddleware { collector, service };
        ready(Ok(middleware))
    }
}

/// Builds a [`MetricsCollector`].
pub struct MetricsCollectorBuilder {
    durations: Option<HistogramVec>,
    errors: Option<CounterVec>,
    prefix: &'static str,
    registry: Option<Registry>,
}

impl MetricsCollectorBuilder {
    /// Use the provided histogram to track request durations.
    pub fn durations(mut self, histogram: HistogramVec) -> Self {
        let desc = histogram.desc();
        let mut descriptions = desc.iter();
        let durations = match descriptions.next() {
            None => panic!("durations histogram has no metrics defined"),
            Some(durations) => durations,
        };
        let mut labels = durations.variable_labels.clone();
        labels.sort();
        if labels != ["method", "path", "status"] {
            panic!(
                "invalid labels defined for the durations histogram: found {:?}",
                labels
            );
        }
        self.durations = Some(histogram);
        self
    }

    /// Use the provided counter to track request errors.
    pub fn errors(mut self, counter: CounterVec) -> Self {
        let desc = counter.desc();
        let mut descriptions = desc.iter();
        let durations = match descriptions.next() {
            None => panic!("errors counter has no metrics defined"),
            Some(durations) => durations,
        };
        let mut labels = durations.variable_labels.clone();
        labels.sort();
        if labels != ["method", "path"] {
            panic!(
                "invalid labels defined for the errors counter: found {:?}",
                labels
            );
        }
        self.errors = Some(counter);
        self
    }

    /// Finalise a `MetricsCollector` build.
    ///
    /// # Panics
    ///
    /// If some metrics are not provided the builder will initialise default metrics.
    /// This method panics in case default metrics are initialised by no [`Registry`] is given.
    ///
    /// This method also panics if registration of the default metrics fails.
    pub fn finish(self) -> MetricsCollector {
        let durations = self.durations.unwrap_or_else(|| {
            let name = format!("{}_request_durations", self.prefix);
            let opts = HistogramOpts::new(name, DEFAULT_METRIC_DURATIONS_DESC);
            let vec = HistogramVec::new(opts, &["method", "path", "status"]).unwrap();
            self.registry
                .as_ref()
                .expect("a registry must be provided for metrics to be auto-created")
                .register(Box::new(vec.clone()))
                .expect("could not register auto-created durations metric");
            vec
        });
        let errors = self.errors.unwrap_or_else(|| {
            let name = format!("{}_request_errors", self.prefix);
            let opts = Opts::new(name, DEFAULT_METRIC_ERRORS_DESC);
            let vec = CounterVec::new(opts, &["method", "path"]).unwrap();
            self.registry
                .as_ref()
                .expect("a registry must be provided for metrics to be auto-created")
                .register(Box::new(vec.clone()))
                .expect("could not register auto-created durations metric");
            vec
        });
        MetricsCollector { durations, errors }
    }

    /// Set the prefix for default metrics names in case they are generated.
    pub fn prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = prefix;
        self
    }

    /// Set the [`Registry`] to register default metrics into in case they are generated.
    pub fn registry(mut self, registry: Registry) -> Self {
        self.registry = Some(registry);
        self
    }
}

impl Default for MetricsCollectorBuilder {
    fn default() -> Self {
        MetricsCollectorBuilder {
            durations: None,
            errors: None,
            prefix: "replisdk",
            registry: None,
        }
    }
}

/// Handle collection of metrics for requests.
pub struct MetricsCollectorMiddleware<S> {
    collector: MetricsCollector,
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsCollectorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let collector = self.collector.clone();
        let method = request.method().as_str().to_owned();
        let path = request
            .match_pattern()
            .unwrap_or_else(|| request.path().to_owned());
        let timer = Instant::now();

        let next = self.service.call(request);
        Box::pin(async move {
            let response = next.await;
            let duration = timer.elapsed().as_secs_f64();

            match &response {
                Ok(response) => {
                    let status = response.response().status();
                    let labels = HashMap::from([
                        ("method", method.as_str()),
                        ("path", path.as_str()),
                        ("status", status.as_str()),
                    ]);
                    collector.durations.with(&labels).observe(duration);
                }
                Err(_) => {
                    let labels =
                        HashMap::from([("method", method.as_str()), ("path", path.as_str())]);
                    collector.errors.with(&labels).inc();
                }
            };
            response
        })
    }
}

#[cfg(test)]
mod tests {
    use actix_web::web::Bytes;
    use actix_web::App;
    use prometheus::CounterVec;
    use prometheus::HistogramOpts;
    use prometheus::HistogramVec;
    use prometheus::Opts;
    use prometheus::Registry;

    use super::MetricsCollector;

    #[actix_web::test]
    async fn collect_metrics() {
        // Create App with middleware.
        let registry = Registry::new();
        let middleware = MetricsCollector::build().registry(registry).finish();
        let app = App::new()
            .wrap(middleware.clone())
            .route("/", actix_web::web::get().to(|| async { "Test Response" }));

        // Send a test request to trigger the middleware.
        let app = actix_web::test::init_service(app).await;
        let request = actix_web::test::TestRequest::get().uri("/").to_request();
        let result = actix_web::test::call_and_read_body(&app, request).await;

        // Inspect the metrics to check for changes.
        assert_eq!(result, Bytes::from_static(b"Test Response"));
        let duration = middleware.durations.with_label_values(&["GET", "/", "200"]);
        assert_eq!(duration.get_sample_count(), 1);
    }

    #[actix_web::test]
    async fn paths_use_placeholders() {
        // Create App with middleware.
        let registry = Registry::new();
        let middleware = MetricsCollector::build().registry(registry).finish();
        let app = App::new().wrap(middleware.clone()).route(
            "/{name}",
            actix_web::web::get().to(|| async { "Test Response" }),
        );

        // Send a test request to trigger the middleware.
        let app = actix_web::test::init_service(app).await;
        let request = actix_web::test::TestRequest::get().uri("/ada").to_request();
        actix_web::test::call_and_read_body(&app, request).await;
        let request = actix_web::test::TestRequest::get()
            .uri("/grace")
            .to_request();
        actix_web::test::call_and_read_body(&app, request).await;

        // Inspect the metrics to check for changes.
        let duration = middleware
            .durations
            .with_label_values(&["GET", "/{name}", "200"]);
        assert_eq!(duration.get_sample_count(), 2);
    }

    #[test]
    #[should_panic(
        expected = "invalid labels defined for the durations histogram: found [\"only\", \"two\"]"
    )]
    fn metrics_labels_checked_for_durations() {
        let registry = Registry::new();
        let histogram =
            HistogramVec::new(HistogramOpts::new("test", "test"), &["only", "two"]).unwrap();
        MetricsCollector::build()
            .durations(histogram)
            .registry(registry)
            .finish();
    }

    #[test]
    #[should_panic(
        expected = "invalid labels defined for the errors counter: found [\"main\", \"too\", \"way\"]"
    )]
    fn metrics_labels_checked_for_errors() {
        let registry = Registry::new();
        let counter = CounterVec::new(Opts::new("test", "test"), &["way", "too", "main"]).unwrap();
        MetricsCollector::build()
            .errors(counter)
            .registry(registry)
            .finish();
    }
}
