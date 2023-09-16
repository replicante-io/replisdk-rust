//! Provides [`actix_web`] utilities to capture and export prometheus metrics.
//!
//! ## Metrics Collection
//!
//! The [`MetricsCollector`] is an [`actix_web`] middleware that generates metrics
//! for requests handlers wrapped by it.
//!
//! ## Metrics Exporter
//!
//! The [`MetricsExporter`] is an [`actix_web::Handler`] object that responds to all requests
//! with the current state of metrics in a [`Registry`](prometheus::Registry),
//! encoded with the [`TextEncoder`](prometheus::TextEncoder).
mod collect;
mod export;

pub use self::collect::MetricsCollector;
pub use self::collect::MetricsCollectorBuilder;
pub use self::collect::MetricsCollectorMiddleware;
pub use self::export::MetricsExporter;
