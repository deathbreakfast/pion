//! Runtime Valence schema registration for Pion.
//!
//! Model schemas are registered by codegen (`OUT_DIR/generated_models.rs` via
//! `build.rs`). Including `*_valence_schema.rs` here would submit a second
//! [`valence::SchemaMetadataInit`] and panic in [`valence::SchemaRegistry`]
//! (duplicate table registration — e.g. `pion_node_reachability`).
//!
//! No trait overlays are registered from this module today.
