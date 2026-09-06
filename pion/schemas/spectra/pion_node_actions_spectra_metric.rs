use spectra_macros::spectra_metric;

spectra_metric! {
    PionNodeActions {
        store: "pion",
        name: "pion_node_actions",
        version: "0.1.0",
        description: "Control-plane node action queue. Labels: action, outcome.",
    }
}
