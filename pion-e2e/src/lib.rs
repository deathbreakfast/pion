//! Matrix-driven **correctness** integration tests for Pion + Parton.
//!
//! The library crate has no public API. Scenarios live under `tests/` and exercise
//! `pion_testkit::ScenarioRunner` against an in-process control plane.
//!
//! # Getting started
//!
//! Prerequisites: workspace checkout; `CARGO_BUILD_JOBS=1` and `CARGO_TARGET_DIR=target-pion`
//! recommended (see `docs/VERIFICATION.md`).
//!
//! ```bash
//! export CARGO_BUILD_JOBS=1
//! export CARGO_TARGET_DIR=target-pion
//! cargo test -p pion-e2e
//! ```
//!
//! Observable outcome: tests pass (exit 0). Optional Docker-gated scenarios require
//! `PION_E2E_DOCKER=1` (see `tests/local_docker.rs`).
//!
//! Harness types: crate `pion-testkit` (`BootstrapSession`, `ScenarioSpec`).

#![deny(missing_docs)]
