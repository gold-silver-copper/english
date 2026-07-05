//! Refresh orchestration: filter -> extract -> emit.
//!
//! There is no persisted state to reconcile: each refresh regenerates the tables
//! from the dump alone. The only guard is the parse-time abort inside the
//! extractors (a wiktextract schema break trips the parse-failure threshold), and
//! the duplicate-key check inside [`generate_tables`]. `--checks-only` measures
//! accuracy without touching the generated tables.

use crate::args::Config;
use crate::bootstrap::generate_tables;
use crate::checks::run_checks;
use crate::extract::{
    extract_irregular_adjectives, extract_irregular_adverbs, extract_irregular_nouns,
    extract_verb_conjugations, filter_english_entries,
};
use std::error::Error;
use std::fs;

/// Run one full data refresh. On success the three generated PHF tables are a
/// pure deterministic function of the (filtered) dump.
pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.generated_dir)?;
    fs::create_dir_all(&config.artifacts_dir)?;

    let filtered_json_path = config.artifacts_dir.join("english_filtered.jsonl");
    // Compare canonicalized paths: a relative/absolute spelling of the SAME file
    // must be detected, or we would truncate the filtered file while reading it.
    let dump_is_filtered = match (
        fs::canonicalize(&config.dump_path),
        fs::canonicalize(&filtered_json_path),
    ) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    if !dump_is_filtered {
        filter_english_entries(&config.dump_path, &filtered_json_path)?;
    } else {
        println!(
            "Reusing filtered dataset at {}",
            filtered_json_path.display()
        );
    }

    if config.checks_only {
        run_checks(&filtered_json_path, &config.artifacts_dir)?;
        println!("Checks-only run: generated tables untouched.");
        return Ok(());
    }

    // Deterministic extraction: each POS is numbered independently from the dump.
    let noun = extract_irregular_nouns(&filtered_json_path)?;
    let verb = extract_verb_conjugations(&filtered_json_path)?;
    let adj = extract_irregular_adjectives(&filtered_json_path)?;
    let adverb = extract_irregular_adverbs(&filtered_json_path)?;

    generate_tables(&noun, &verb, &adj, &adverb, &config.generated_dir)?;

    println!(
        "Refresh complete. Tables regenerated in {}.",
        config.generated_dir.display()
    );
    Ok(())
}
