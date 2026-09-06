use spectra_macros::spectra_metric;

spectra_metric! {
    PionServerBootSteps {
        store: "pion",
        name: "pion_server_boot_steps",
        version: "0.1.0",
        description: "pion-server bootstrap step outcomes. Labels: step, outcome.",
    }
}
