use spectra_macros::spectra_schema;

spectra_schema! {
    PionHandoffLog {
        store: "pion",
        table: "pion_handoff_log",
        version: "0.1.0",
        description: "Handoff directive deliver, ack, and collect trace.",
        fields: [
            directive_id: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            node_id: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            phase: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            reason: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            message: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
        ],
    }
}
