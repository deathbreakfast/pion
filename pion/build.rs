//! Valence model codegen for `schemas/**/*_valence_schema.rs` (recursive).

// Build scripts must emit `cargo:` directives via println!.
#![allow(clippy::print_stdout)]

fn main() {
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        // Cargo always sets OUT_DIR for build scripts; exit if the environment is broken.
        std::process::exit(1);
    };
    let out_dir = std::path::PathBuf::from(out_dir);
    // Point at `schemas/valence` so discovery works with published (non-recursive)
    // uf-valence-codegen as well as the recursive local scanner.
    let schemas_dir = std::path::PathBuf::from("schemas/valence");

    // Cargo build-script directive (not application logging).
    println!("cargo:rerun-if-changed=schemas/");

    if let Err(error) = valence_codegen::generate_models(&valence_codegen::CodegenConfig {
        schemas_dir,
        out_dir,
        file_suffix: "_valence_schema.rs",
        trait_file_suffix: "_valence_trait.rs",
    }) {
        panic!("valence codegen failed: {error}");
    }
}
