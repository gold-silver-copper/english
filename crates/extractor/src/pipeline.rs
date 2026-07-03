use crate::args::Config;
use crate::bootstrap::{generate_tables, load_locks, save_locks};
use crate::checks::run_checks;
use crate::extract::{
    extract_irregular_adjectives, extract_irregular_nouns, extract_verb_conjugations,
    filter_english_entries,
};
use crate::registry::check_immutability;
use std::error::Error;
use std::fs;

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.generated_dir)?;
    fs::create_dir_all(&config.artifacts_dir)?;
    fs::create_dir_all(&config.assignments_dir)?;

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

    if config.run_checks {
        run_checks(&filtered_json_path, &config.artifacts_dir)?;
    }
    if config.checks_only {
        println!("Checks-only run: lockfiles and generated tables untouched.");
        return Ok(());
    }

    // Load the committed assignment lockfiles (empty on first run). We keep a
    // pristine copy to diff against after resolution, so a refresh can report any
    // previously-published key whose meaning would change.
    let (before_noun, before_verb, before_adj) = load_locks(&config.assignments_dir)?;
    let (mut noun_lock, mut verb_lock, mut adj_lock) = load_locks(&config.assignments_dir)?;

    let verb_reap = extract_verb_conjugations(&filtered_json_path, &mut verb_lock, &config.data_date)?;
    let noun_reap = extract_irregular_nouns(&filtered_json_path, &mut noun_lock, &config.data_date)?;
    let adj_reap = extract_irregular_adjectives(&filtered_json_path, &mut adj_lock, &config.data_date)?;

    // A skipped reap means the dump looks partial/corrupt (it would have retired
    // an implausible share of a part-of-speech). Writing lockfiles from such a
    // dump would ship a stale/mixed state that check-registry cannot flag (the
    // absent rows were never tombstoned) — so this is a hard abort, not a note,
    // unless the operator explicitly accepts it with --allow-partial-refresh.
    let skipped: Vec<&str> = [("noun", noun_reap), ("verb", verb_reap), ("adj", adj_reap)]
        .iter()
        .filter(|(_, r)| r.skipped)
        .map(|(pos, _)| *pos)
        .collect();
    if !skipped.is_empty() && !config.allow_partial_refresh {
        return Err(format!(
            "refusing to write lockfiles: the dump looks partial/corrupt (mass-absence reap \
             guard tripped for: {}). Nothing was written. If this dump is genuinely correct, \
             re-run with --allow-partial-refresh.",
            skipped.join(", ")
        )
        .into());
    }

    // Surface resolution notes (drift re-matches, tombstones, ambiguities).
    for note in noun_lock
        .notes
        .iter()
        .chain(verb_lock.notes.iter())
        .chain(adj_lock.notes.iter())
    {
        println!("note: {note}");
    }

    // Report (do not abort) immutability violations vs. the committed lock; the
    // `check-registry` xtask is the hard CI gate.
    let mut violations = Vec::new();
    violations.extend(check_immutability(&before_noun, &noun_lock));
    violations.extend(check_immutability(&before_verb, &verb_lock));
    violations.extend(check_immutability(&before_adj, &adj_lock));
    if !violations.is_empty() {
        eprintln!(
            "\nWARNING: {} immutability violation(s) detected vs the committed lock:",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {v}");
        }
        eprintln!(
            "Review the lock diff carefully; `cargo xtask check-registry` will fail CI on these.\n"
        );
    }

    // Strict structural validation (suffix>=1, anchor<->fields, no emit-key
    // collisions) is a hard abort — a structurally broken lock must never be written.
    let invalid: Vec<String> = noun_lock
        .validate()
        .into_iter()
        .chain(verb_lock.validate())
        .chain(adj_lock.validate())
        .collect();
    if !invalid.is_empty() {
        return Err(format!(
            "refusing to write lockfiles — {} structural violation(s):\n  - {}",
            invalid.len(),
            invalid.join("\n  - ")
        )
        .into());
    }

    save_locks(&config.assignments_dir, &noun_lock, &verb_lock, &adj_lock)?;
    generate_tables(&noun_lock, &verb_lock, &adj_lock, &config.generated_dir)?;

    println!(
        "Refresh complete. Lockfiles updated in {}, tables in {}.",
        config.assignments_dir.display(),
        config.generated_dir.display()
    );
    Ok(())
}
