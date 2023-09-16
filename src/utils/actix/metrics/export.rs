use std::future::Ready;

use actix_web::http::header::CONTENT_TYPE;
use actix_web::web;
use actix_web::Handler;
use actix_web::HttpResponse;
use actix_web::Resource;
use actix_web::ResponseError;
use prometheus::Encoder;
use prometheus::Registry;
use prometheus::TextEncoder;

/// ActixWeb [`Handler`] to export metrics from a [`Registry`].
#[derive(Clone, Debug)]
pub struct MetricsExporter {
    registry: Registry,
}

impl MetricsExporter {
    /// Create a `MetricsExporter` that will export metrics from the given [`Registry`].
    pub fn new(registry: Registry) -> MetricsExporter {
        MetricsExporter { registry }
    }

    /// Wrap a `MetricsExporter` for the given [`Registry`] into a ready to go [`Resource`].
    ///
    /// The [`Resource`] will handle `GET` requests to the `/metrics` path.
    pub fn simple(registry: Registry) -> Resource {
        let handler = MetricsExporter::new(registry);
        web::resource("/metrics").route(web::get().to(handler))
    }
}

impl Handler<()> for MetricsExporter {
    type Output = HttpResponse;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        let metrics = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        let response = match encoder.encode(&metrics, &mut buffer) {
            Ok(()) => HttpResponse::Ok()
                .append_header((CONTENT_TYPE, encoder.format_type()))
                .body(buffer),
            Err(error) => {
                let error = anyhow::anyhow!(error);
                let error = error.context(anyhow::anyhow!("unable to encode metrics"));
                let error = crate::utils::actix::error::Error::from(error);
                error.error_response()
            }
        };
        std::future::ready(response)
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;
    use actix_web::web;
    use actix_web::web::Bytes;
    use actix_web::App;
    use prometheus::Counter;
    use prometheus::Registry;

    use super::MetricsExporter;

    const EXPECTED_METRICS: &[u8] =
        b"# HELP metric test metric to encode\n# TYPE metric counter\nmetric 0\n";

    fn make_registry() -> Registry {
        let registry = Registry::new();
        let counter = Counter::new("metric", "test metric to encode").unwrap();
        registry.register(Box::new(counter)).unwrap();
        registry
    }

    fn make_request() -> TestRequest {
        actix_web::test::TestRequest::get().uri("/metrics")
    }

    #[actix_web::test]
    async fn metrics_exporter_handler() {
        // Configure metrics exporter.
        let registry = make_registry();
        let handler = MetricsExporter::new(registry);

        // Send a request for metrics and check the returned data.
        let app = App::new().service(web::resource("/metrics").route(web::get().to(handler)));
        let app = actix_web::test::init_service(app).await;
        let request = make_request().to_request();
        let result = actix_web::test::call_and_read_body(&app, request).await;
        assert_eq!(result, Bytes::from_static(EXPECTED_METRICS));
    }

    #[actix_web::test]
    async fn metrics_exporter_resource() {
        // Configure metrics exporter.
        let registry = make_registry();
        let resource = MetricsExporter::simple(registry);

        // Send a request for metrics and check the returned data.
        let app = App::new().service(resource);
        let app = actix_web::test::init_service(app).await;
        let request = make_request().to_request();
        let result = actix_web::test::call_and_read_body(&app, request).await;
        assert_eq!(result, Bytes::from_static(EXPECTED_METRICS));
    }
}
