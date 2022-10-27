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
//! * Enable all logic related to Replicante Platforms, typically needed to implement Platforms.
//! * Enable specific parts such as Platform models, typically needed to implement Platform clients.
//!
//! Cargo features follow a standard naming convention: `${AREA}-${FEATURE}`,
//! with an `${AREA}` feature also available to enable all `${AREA}-*` features.
//!
//! By default the SDK provides little to nothing and requires you to opt into what you need:
//!
//! ## Platforms
//!
//! The following features are available for the platforms area:
//!
//! * `platform-models`: Enable definitions of (infrastructure) platform data models.
//!
//! # The experimental crate
//!
//! While the SDK is evolving and the ecosystem growing it is essential to balance
//! speed of change with stability.
//! Support for experimental features or changes are made into a dedicated
//! `replisdk-experimental` crate.
//! This crate, as the name suggests, has no stability guarantees:
//!
//! * Added features may never become stable and could be dropped without replacement.
//! * Breaking changes can be made across any version, so the crate will likely never reach 1.0.
//!
//! [Rust Lang]: https://www.rust-lang.org/
#![deny(missing_docs)]

#[cfg(any(feature = "platform-models"))]
pub mod platform;
