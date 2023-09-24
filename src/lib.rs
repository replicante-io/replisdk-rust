//! The Replicante project is a combination of multiple applications, tools and integrations.
//! This SDK aids development of the Replicante ecosystem when using [Rust Lang].
//!
//! # One SDK, many uses
//!
//! This SDK is provided as one package but is intended to support development of
//! the many different applications, tools and integrations that are the Replicante ecosystem.
//!
//! A single SDK crate provides the community with a clear starting point and provides users
//! with a more consistent experience across ecosystem components.
//!
//! # Areas of the SDK and cargo features
//!
//! But using a single crate for everything can lead to bloat and needless overhead.
//! To avoid including undesired logic or dependencies the SDK uses cargo features.
//!
//! The SDK and its features are organised by area of the Replicante ecosystem.
//! Cargo features can be very granular and enable only small/specific features or
//! they can enable larger areas of the SDK to enable more complete use cases.
//!
//! For example features can be used to:
//!
//! - Enable all logic related to Replicante Platforms, typically needed to implement Platforms.
//! - Enable specific parts such as Platform models, typically needed to implement Platform clients.
//!
//! Cargo features follow a standard naming convention: `${AREA}-${FEATURE}`,
//! with an `${AREA}` feature also available to enable all `${AREA}-*` features.
//!
//! By default the SDK provides little to nothing and requires you to opt into what you need:
//!
//! ## Agents
//!
//! The following features are available for the agents area:
//!
//! - `agent-framework`: Enable tools to implement Replicante Agents.
//! - `agent-models`: Enable definitions of (Replicante) agent data models.
//!
//! ## Context
//!
//! The `context` feature enables a general purpose container to carry scoped values around.
//! Different frameworks in the SDK use contexts to carry request specific information.
//!
//! ## Platforms
//!
//! The following features are available for the platforms area:
//!
//! - `platform-framework`: Enable tools to implement Replicante Platform servers.
//! - `platform-framework_actix`: Enable utilities to run `IPlatform`s in `actix_web` servers.
//! - `platform-models`: Enable definitions of (infrastructure) platform data models.
//!
//! ## RepliCore
//!
//! The following features are available for the Replicante Core area:
//!
//! - `replicore-models`: Enable definitions of Replicante Core data and API models.
//!
//! ## Runtime
//!
//! The runtime provides utilities to manage general features and needs of the process lifecycle.
//!
//! - `runtime-actix_builder`: Enable Actix Web server runtime configuration utilities.
//! - `runtime-shutdown`: Enable tools to manage process shutdown on error or at user's request.
//! - `runtime-shutdown_acitx`: Enable process shutdown extension to watch for `actix_web` servers.
//! - `runtime-telemetry`: Enable utilities to initialise runtime telemetry of the process.
//! - `runtime-tokio_conf`: Enable tokio runtime configuration utilities.
//!
//! ## Testing
//!
//! - `test-fixture`: Enable test fixtures defined by other features.
//!
//! ## Utilities
//!
//! A configurable collection of various utilities and code for common tasks.
//!
//! - `utils-actix_error`: An `actix_web` error type that works with `anyhow::Error`.
//! - `utils-actix_metrics`: Collect metrics about processed requests and an exporter all metrics.
//! - `utils-error_json`: Utility function to encode an error into a JSON object.
//! - `utils-error_slog`: Standard way to log errors as slog key/value pairs.
//! - `utils-metrics`: Utilities to introspect applications and libraries with metrics more easley.
//! - `utils-trace`: Utilities to introspect applications and libraries with traces more easley.
//!
//! # The experimental crate
//!
//! While the SDK is evolving and the ecosystem growing it is essential to balance
//! speed of change with stability.
//! Support for experimental features or changes are made into a dedicated
//! `replisdk-experimental` crate.
//! This crate, as the name suggests, has no stability guarantees:
//!
//! - Added features may never become stable and could be dropped without replacement.
//! - Breaking changes can be made across any version, so the crate will likely never reach 1.0.
//!
//! [Rust Lang]: https://www.rust-lang.org/
#![deny(missing_docs)]

#[cfg(any(feature = "agent-framework", feature = "agent-models"))]
pub mod agent;

#[cfg(feature = "context")]
pub mod context;

#[cfg(feature = "replicore-models")]
pub mod core;

#[cfg(any(feature = "platform-framework", feature = "platform-models"))]
pub mod platform;

#[cfg(any(
    feature = "runtime-actix_builder",
    feature = "runtime-shutdown",
    feature = "runtime-telemetry",
    feature = "runtime-tokio_conf",
))]
pub mod runtime;

#[cfg(any(feature = "utils-actix_error", feature = "utils-error_slog"))]
pub mod utils;
