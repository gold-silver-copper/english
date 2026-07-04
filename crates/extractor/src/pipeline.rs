//! Refresh orchestration: filter -> extract -> resolve -> validate -> write.
//!
//! The ordering of the guard rails is the point of this module. Everything
//! that can reject a refresh does so BEFORE `save_locks`/`generate_tables`
//! touch the working tree:
//!
//! 1. partial-dump abort — a skipped mass-absence reap means the dump is
//!    suspect; writing would leave stale rows active that `check-registry`
//!    could never flag (they were never tombstoned). Overridable only by the
//!    explicit `--allow-partial-refresh` flag.
//! 2. structural validation (`Lock::validate`) — a malformed lock is never
//!    written, full stop.
//!
//! Immutability violations vs. the committed lock are deliberately only a
//! WARNING here: the refresh must be able to produce a diff a human then
//! reviews, and `cargo xtask check-registry` is the hard gate that keeps such
//! a diff from merging. Failing the refresh itself would make legitimate
//! relocks impossible to even generate.

use crate::args::Config;
use crate::bootstrap::{generate_tables, load_locks, save_locks};
use crate::checks::run_checks;
use crate::extract::{
    extract_irregular_adjectives, extract_irregular_nouns, extract_verb_conjugations,
    filter_english_entries,
};
use crate::helpers::Pos;
use crate::registry::{Overrides, check_immutability, save_proposals};
use std::error::Error;
use std::fs;

/// Run one full data refresh. See the module docs for the guard-rail ordering;
/// on success the lockfiles and all generated tables are consistent with each
/// other by construction (same in-memory locks feed both writers).
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
    // pristine clone to diff against after resolution, so a refresh can report any
    // previously-published key whose meaning would change.
    let (mut noun_lock, mut verb_lock, mut adj_lock) = load_locks(&config.assignments_dir)?;
    let (before_noun, before_verb, before_adj) =
        (noun_lock.clone(), verb_lock.clone(), adj_lock.clone());

    // Reviewed rematch sanctions: the ONLY way a retired pairing is re-made.
    let overrides = Overrides::load(config.assignments_dir.join("overrides.csv"))?;
    noun_lock.set_overrides(&overrides);
    verb_lock.set_overrides(&overrides);
    adj_lock.set_overrides(&overrides);

    let verb_reap =
        extract_verb_conjugations(&filtered_json_path, &mut verb_lock, &config.data_date)?;
    let noun_reap =
        extract_irregular_nouns(&filtered_json_path, &mut noun_lock, &config.data_date)?;
    let adj_reap =
        extract_irregular_adjectives(&filtered_json_path, &mut adj_lock, &config.data_date)?;

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
             guard tripped for: {}). Nothing was written. --allow-partial-refresh proceeds \
             WITHOUT reaping (vanished lemmas stay active with their last-seen forms — it does \
             not mean 'trust this dump'); if this many lemmas really did vanish upstream, raise \
             REAP_MAX_FRACTION deliberately and review the tombstone diff.",
            skipped.join(", ")
        )
        .into());
    }

    // Surface resolution notes (override applications, tombstones, ambiguities).
    for note in noun_lock
        .notes
        .iter()
        .chain(verb_lock.notes.iter())
        .chain(adj_lock.notes.iter())
    {
        println!("note: {note}");
    }

    // Every override must have been APPLIED by this refresh (the dump was just
    // consulted, so a still-pending or mismatched override is stale). Hard abort:
    // writing the lock while overrides.csv contradicts it would leave the
    // sanction file lying about what it sanctions.
    let stale: Vec<String> = overrides
        .check(&noun_lock, Pos::Noun, true)
        .into_iter()
        .chain(overrides.check(&verb_lock, Pos::Verb, true))
        .chain(overrides.check(&adj_lock, Pos::Adj, true))
        .collect();
    if !stale.is_empty() {
        return Err(format!(
            "refusing to write lockfiles — {} override problem(s):\n  - {}",
            stale.len(),
            stale.join("\n  - ")
        )
        .into());
    }

    // Rematches the resolver refused to guess at: recorded for human review.
    // Approving one = copying its first five columns into overrides.csv.
    let proposals: Vec<_> = noun_lock
        .proposals
        .iter()
        .chain(verb_lock.proposals.iter())
        .chain(adj_lock.proposals.iter())
        .cloned()
        .collect();
    if !proposals.is_empty() {
        let path = config.artifacts_dir.join("pending-rematches.csv");
        save_proposals(&proposals, &path)?;
        println!(
            "{} rematch proposal(s) written to {} — review and sanction real matches via \
             data/assignments/overrides.csv.",
            proposals.len(),
            path.display()
        );
    }

    // Report (do not abort) immutability violations vs. the committed lock; the
    // `check-registry` xtask is the hard CI gate.
    let mut violations = Vec::new();
    violations.extend(check_immutability(&before_noun, &noun_lock, &overrides));
    violations.extend(check_immutability(&before_verb, &verb_lock, &overrides));
    violations.extend(check_immutability(&before_adj, &adj_lock, &overrides));
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
