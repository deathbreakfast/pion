use spectra_macros::spectra_schema;

spectra_schema! {
    PionNodeActionLog {
        store: "pion",
        table: "pion_node_action_log",
        version: "0.1.0",
        description: "Control-plane node action queue lifecycle trace.",
        fields: [
            command_id: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            node_id: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            action_kind: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            correlation_key: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            sequence: {
                r#type: i64,
                classification: { pii: false, safe_for_console: true },
            },
            attempt: {
                r#type: i64,
                classification: { pii: false, safe_for_console: true },
            },
            outcome: {
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
