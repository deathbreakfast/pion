use spectra_macros::spectra_metric;

spectra_metric! {
    PionHandoffDirectives {
        store: "pion",
        name: "pion_handoff_directives",
        version: "0.1.0",
        description: "Handoff directive deliver/ack/collect outcomes. Labels: phase, outcome.",
    }
}
