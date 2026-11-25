//! Build-generated Valence models (`Pion*`) from `schemas/valence/*_valence_schema.rs`.
//!
//! Do not edit the included file by hand; regenerate via `build.rs` / valence-codegen.

#![allow(
    dead_code,
    unused_imports,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::restriction
)]

#[cfg(feature = "runtime")]
use valence::privacy_policies::common::{AUTHENTICATED, SYSTEM_ONLY};

#[cfg(feature = "runtime")]
use crate::privacy_policies::PION_OPERATE_CLOUD_RESOURCES;

#[cfg(feature = "runtime")]
include!(concat!(env!("OUT_DIR"), "/generated_models.rs"));
