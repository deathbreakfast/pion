//! Shared helpers for Pion instrumentation tests.
// Test helpers use `.expect()` / `.unwrap()` for brevity on fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::missing_panics_doc)]
use spectra_core::RecordingSink;

/// Assert exactly one metric counter sample matches `name` and label set.
pub fn assert_counter(sink: &RecordingSink, name: &str, labels: &[(&str, &str)]) {
    let hits = sink.recorded_counters_matching(name, labels);
    assert!(
        !hits.is_empty(),
        "expected counter {name} with labels {labels:?}, got {:?}",
        sink.counters()
    );
}

/// Assert an event row contains `field == expected` for `table`.
pub fn assert_event_field(sink: &RecordingSink, table: &str, field: &str, expected: &str) {
    let events = sink.recorded_events_for(table);
    assert!(
        events.iter().any(|e| {
            e.fields
                .get(field)
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == expected)
        }),
        "expected event {table}.{field}={expected:?}, got {events:?}"
    );
}
