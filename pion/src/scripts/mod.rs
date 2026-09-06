//! Maintenance scripts for the Pion control-plane domain.
//!
//! Each script is a plain `async fn` (see [`migrate_gluon_control_plane_tables`]) callable directly
//! by operators or wrapped by a downstream scheduler. Upstream Phase-1 does not depend on Chronon.
//!
//! # Available scripts
//!
//! | Script | Purpose |
//! |--------|---------|
//! | [`migrate_gluon_control_plane_tables`] | **No-op** Chronon-stable job name (legacy Surreal table copy removed) |
//!
//! # Testing
//!
//! Integration tests live under `pion/tests/` (for example
//! `migrate_gluon_control_plane_tables_integration.rs`) and call the plain `async fn` entrypoints
//! directly.

pub mod migrate_gluon_control_plane_tables;
