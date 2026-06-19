use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));

    let mut required = vec![
        "generated/noun_phf.rs",
        "generated/adj_phf.rs",
        "generated/verb_phf.rs",
    ];
    // Feature-gated data tables are only required when their feature is enabled.
    if env::var_os("CARGO_FEATURE_SENSES").is_some() {
        required.push("generated/variants_phf.rs");
    }
    if env::var_os("CARGO_FEATURE_DICTIONARY").is_some() {
        required.push("generated/dictionary_phf.rs");
    }

    for relative_path in required {
        let absolute_path = manifest_dir.join(relative_path);
        println!("cargo:rerun-if-changed={}", absolute_path.display());

        assert!(
            absolute_path.exists(),
            "missing generated data file: {}. Run `cargo xtask refresh-data --dump /path/to/rawwiki.jsonl`.",
            absolute_path.display()
        );
    }
}
