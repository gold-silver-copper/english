use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));

    for relative_path in [
        "generated/noun_phf.rs",
        "generated/adj_phf.rs",
        "generated/verb_phf.rs",
    ] {
        let absolute_path = manifest_dir.join(relative_path);
        println!("cargo:rerun-if-changed={}", absolute_path.display());

        assert!(
            absolute_path.exists(),
            "missing generated data file: {}. Run `cargo xtask refresh-data --dump /path/to/rawwiki.jsonl`.",
            absolute_path.display()
        );
    }
}
