<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Agent framework: action execution.
- Agent framework: definition of store for agents to persist data into.
- Agent framework: node information trait.
- Agent framework: reusable process initialisation logic.
- Agent framework: schedule and list actions.
- Agent framework: wellknown `agent.replicante.io/test.*` actions.
- Error type to bridge anyhow and `actix-web` response rendering.
- Platform API models for cluster discovery.
- Platform deprovisioning models.
- Platform framework: `actix-web` service wrapper.
- Platform framework: platform trait definition and default context.
- Platform models for Core API.
- Platform provisioning models.
- Prometheus metrics collection and export utilities for the `actix-web` framework.
- RepliCore models: authentication and authorisation related models.
- RepliCore models: cluster specification model.
- RepliCore models: namespace models.
- RepliCore models: orchestrator actions.
- Runtime actix-web server configuration.
- Runtime telemetry initialisation utilities.
- Runtime utility to manage async process and shutdown.
- Store Agent models.
- Utilities to encode and decode data types into or from strings.
- Utilities to introspect applications and libraries more easley.

### Changed

- **BREAKING**: Add `node_class` and `node_group` to `ClusterDiscoveryNode`.
- Require Rust `1.70` or later.

## 0.1.0 - 2022-10-28

### Added

- Platform models for cluster discovery.

[Unreleased]: https://github.com/replicante-io/replisdk-rust/compare/v0.1.0...HEAD
