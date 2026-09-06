//! Asserts the hand-mirrored constants in `pion_spectra_topics` stay in sync with the real
//! `spectra_schema!` / `spectra_metric!` registrations linked into the `pion` crate.
//!
//! Linking `pion` with its `runtime` feature (dev-only here) is what actually submits
//! `pion::spectra_schemas` to the `inventory`-backed `spectra_core::SchemaRegistry`; without it
//! the registry would be empty and this test would vacuously pass. If `pion` renames, adds, or
//! removes a schema/metric without a matching update here, these assertions fail.

use spectra_core::{LoggingKind, SchemaRegistry};

// Forces `pion`'s `spectra_schemas` module (and therefore every `spectra_schema!` /
// `spectra_metric!` registration) to be linked into this test binary.
#[allow(unused_imports)]
use pion as _pion_links_spectra_schemas;

#[test]
fn event_constants_match_registered_pion_event_schemas() {
    let registry = SchemaRegistry::global();
    for &name in pion_spectra_topics::event::ALL {
        let schema = registry
            .get_schema(name)
            .unwrap_or_else(|| panic!("expected pion event schema `{name}` to be registered"));
        assert_eq!(schema.store, pion_spectra_topics::STORE);
        assert_eq!(schema.logging_kind, LoggingKind::Event);
    }
}

#[test]
fn metric_constants_match_registered_pion_metric_schemas() {
    let registry = SchemaRegistry::global();
    for &name in pion_spectra_topics::metric::ALL {
        let schema = registry
            .get_schema(name)
            .unwrap_or_else(|| panic!("expected pion metric schema `{name}` to be registered"));
        assert_eq!(schema.store, pion_spectra_topics::STORE);
        assert_eq!(schema.logging_kind, LoggingKind::Metric);
    }
}

#[test]
fn every_pion_store_schema_is_covered_by_this_crate() {
    let registry = SchemaRegistry::global();
    let known: std::collections::BTreeSet<&str> = pion_spectra_topics::event::ALL
        .iter()
        .copied()
        .chain(pion_spectra_topics::metric::ALL.iter().copied())
        .collect();
    for name in registry.list_schemas() {
        let Some(schema) = registry.get_schema(name) else {
            continue;
        };
        if schema.store != pion_spectra_topics::STORE {
            continue;
        }
        assert!(
            known.contains(name),
            "pion registers `{name}` under store `{}` but pion_spectra_topics does not mirror it",
            pion_spectra_topics::STORE
        );
    }
}
