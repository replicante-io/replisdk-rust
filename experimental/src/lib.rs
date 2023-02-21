//! Experimental features and additions to the Repicante SDK for rust ([`replisdk`]).
//!
//! # Stable SDK
//!
//! As the name implies this crate focuses on newer, unstable elements in the SDK.
//!
//! You should first start by checking out the more stable [`replisdk`] crate and
//! come back with a better understanding of the Rust SDK and how it is structured.
//!
//! # Experimental features
//!
//! By default the experimental SDK provides little to nothing and, just the stable SDK,
//! requires you to opt into what you need:
//!
//! ## Platforms
//!
//! The following features are available for the platforms area:
//!
//! * `platform-templates`: Enable tools to mange node templates in Replicante Platform servers.
#![deny(missing_docs)]

#[cfg(feature = "platform-templates")]
pub mod platform;
