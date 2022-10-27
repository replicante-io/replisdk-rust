# Replicante Rust SDK

The Replicante project is a combination of multiple applications and integrations.
This SDK aids development of tools and processes for the Replicante ecosystem in rust.

A single SDK crate provides the community with a clear starting point and provides users
with a more consistent experience.

## Documentation

The Rust SDK is documented using `rustdoc` and available on [docs.rs/replisdk].

## Areas of the SDK

* Agents: Replicante Agents enable integration with stateful processes and provide common
  logic and actions.
  This area of the SDK focuses on easily build feature-complete agents.

* Core: Replicante Core offers a public interface and data models.
  This area of the SDK focuses on integrating this Core or building additional components.

* Platforms: Platform integrations allow Replicante Core to manage the infrastructure running
  clusters without having to know the details around it.
  This area of the SDK focuses on easily building platforms integrations to quickly provision
  clusters anywhere.

* Various utilities: as with any project all Replicante ecosystem components share some logic and
  ancillary features (for example: process setup, observability, inter-process communication, ...).

### Features list

But when building agents you don't want to the platform SDK code bloating the final binaries.
Cargo features are used to provide a single SDK crate without including needless logic.

All SDK features are gated so nothing is provided by default and you have to opt into what you need.
Below is a summary of available cargo features:

* `platform` - Platform SDK features:
  * `platform-models` - Data models used to interact with platform integrations.

## Experimental features and changes

While the SDK is evolving and the ecosystem growing it is essential to balance speed of change
with stability.

The most experimental stuff and sometimes the newer features are made available through
a dedicated `replisdk-experimental` crate.
This isolates SDK users from the most likely to change features of the SDK.

Over time the features in `replisdk-experimental` are meant to be either deprecated or stabilised
into `replisdk`.

[docs.rs/replisdk]: https://docs.rs/replisdk/
