//! One-time bootstrap and shared table generation.
//!
//! The library ships with pre-generated PHF tables but no lockfile. [`seed`]
//! parses those committed tables and freezes today's exact keys into the
//! assignment lockfiles, so every previously-published key (`lie2`, `die2`,
//! `bad3`, ...) is grandfathered byte-for-byte. Because no Wiktionary dump is
//! consulted here, bootstrapped rows are anchored on their form signature
//! (`sig:<forms>`); later `refresh-data` runs enrich them with the stronger
//! etymology / senseid / QID anchors as those become available.
//!
//! [`generate_tables`] is the single lock -> PHF path, shared by both `seed`
//! (round-trips the existing tables, proving losslessness) and the refresh
//! pipeline (emits tables from freshly-resolved locks).

use crate::file_generation::{write_adjectives_phf, write_nouns_phf, write_verbs_phf};
use crate::helpers::Pos;
use crate::registry::{Lock, LockRow, Status};
use std::error::Error;
use std::fs;
use std::path::Path;

/// A parsed table row: a key and its inflected form columns.
pub type KeyForms = (String, Vec<String>);

/// Extract `(key, form-columns)` pairs from a generated `*_phf.rs` file by reading
/// the quoted string literals on each `"key" => ...,` line. The first quoted
/// field is the key; the rest are the inflected forms in column order.
pub fn parse_phf_pairs(path: impl AsRef<Path>) -> Result<Vec<KeyForms>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('"') || !line.contains("=>") {
            continue;
        }
        let fields = quoted_fields(line);
        if fields.len() < 2 {
            continue;
        }
        let key = fields[0].clone();
        let forms = fields[1..].to_vec();
        out.push((key, forms));
    }
    Ok(out)
}

/// Collect every double-quoted segment on a line.
fn quoted_fields(line: &str) -> Vec<String> {
    line.split('"')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, s)| s.to_string())
        .collect()
}

fn strip_trailing_digits(word: &str) -> (&str, u32) {
    let base = word.trim_end_matches(|c: char| c.is_ascii_digit());
    let digits = &word[base.len()..];
    let suffix = digits.parse::<u32>().unwrap_or(1);
    (base, suffix)
}

/// Build a lock for one part of speech directly from existing `(key, forms)` pairs.
pub fn lock_from_pairs(pairs: &[KeyForms], pos: Pos, date: &str) -> Lock {
    let mut lock = Lock::new();
    for (key, forms) in pairs {
        let (lemma, suffix) = strip_trailing_digits(key);
        let anchor = format!("sig:{}", forms.join("|"));
        lock.insert_row(LockRow {
            lemma: lemma.to_string(),
            pos,
            suffix,
            anchor,
            qid: None,
            sid: None,
            etym: None,
            status: Status::Active,
            first_seen: date.to_string(),
            last_seen: date.to_string(),
            forms: forms.clone(),
            gloss: None,
        });
    }
    lock
}

fn lock_path(assignments_dir: &Path, pos: Pos) -> std::path::PathBuf {
    assignments_dir.join(format!("{}.lock.csv", pos.as_str()))
}

/// Generate all PHF tables (noun/verb/adj + variants) from resolved locks.
/// This is the single source-of-truth -> tables path.
pub fn generate_tables(
    noun_lock: &Lock,
    verb_lock: &Lock,
    adj_lock: &Lock,
    generated_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    // Pre-emit guard: refuse to write a table with colliding keys, which would
    // otherwise blow up only as an opaque `phf_map!` duplicate-key compile error.
    let collisions: Vec<String> = [
        (Pos::Noun, noun_lock),
        (Pos::Verb, verb_lock),
        (Pos::Adj, adj_lock),
    ]
    .iter()
    .flat_map(|(pos, lock)| lock.emit_collisions(*pos))
    .collect();
    if !collisions.is_empty() {
        return Err(format!(
            "refusing to generate tables — {} duplicate emit key(s):\n  - {}",
            collisions.len(),
            collisions.join("\n  - ")
        )
        .into());
    }

    // Nouns: one plural column.
    let noun_emit = noun_lock.emittable(Pos::Noun);
    let noun_pairs: Vec<(String, String)> = noun_emit
        .iter()
        .map(|(k, f)| (k.clone(), col(f, 0)))
        .collect();
    write_nouns_phf(noun_pairs, generated_dir.join("noun_phf.rs"))?;

    // Verbs: four columns.
    let verb_emit = verb_lock.emittable(Pos::Verb);
    let verb_pairs: Vec<(String, (String, String, String, String))> = verb_emit
        .iter()
        .map(|(k, f)| (k.clone(), (col(f, 0), col(f, 1), col(f, 2), col(f, 3))))
        .collect();
    write_verbs_phf(verb_pairs, generated_dir.join("verb_phf.rs"))?;

    // Adjectives: two columns.
    let adj_emit = adj_lock.emittable(Pos::Adj);
    let adj_pairs: Vec<(String, (String, String))> = adj_emit
        .iter()
        .map(|(k, f)| (k.clone(), (col(f, 0), col(f, 1))))
        .collect();
    write_adjectives_phf(adj_pairs, generated_dir.join("adj_phf.rs"))?;

    Ok(())
}

fn col(forms: &[String], i: usize) -> String {
    forms.get(i).cloned().unwrap_or_default()
}

/// One-time seed: parse committed tables -> write lockfiles -> regenerate tables
/// from those locks (round-trip; should be byte-identical to what was committed).
pub fn seed(generated_dir: &Path, assignments_dir: &Path, date: &str) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(assignments_dir)?;

    let noun_pairs = parse_phf_pairs(generated_dir.join("noun_phf.rs"))?;
    let verb_pairs = parse_phf_pairs(generated_dir.join("verb_phf.rs"))?;
    let adj_pairs = parse_phf_pairs(generated_dir.join("adj_phf.rs"))?;

    println!(
        "Parsed existing tables: {} nouns, {} verbs, {} adjectives",
        noun_pairs.len(),
        verb_pairs.len(),
        adj_pairs.len()
    );

    let noun_lock = lock_from_pairs(&noun_pairs, Pos::Noun, date);
    let verb_lock = lock_from_pairs(&verb_pairs, Pos::Verb, date);
    let adj_lock = lock_from_pairs(&adj_pairs, Pos::Adj, date);

    noun_lock.save(lock_path(assignments_dir, Pos::Noun))?;
    verb_lock.save(lock_path(assignments_dir, Pos::Verb))?;
    adj_lock.save(lock_path(assignments_dir, Pos::Adj))?;
    println!("Wrote lockfiles to {}", assignments_dir.display());

    // Round-trip: regenerate the tables from the locks we just wrote.
    generate_tables(&noun_lock, &verb_lock, &adj_lock, generated_dir)?;
    println!(
        "Regenerated tables from locks in {} (should be a no-op diff)",
        generated_dir.display()
    );

    Ok(())
}

/// Load the three per-pos lockfiles (each empty if missing).
pub fn load_locks(assignments_dir: &Path) -> Result<(Lock, Lock, Lock), Box<dyn Error>> {
    Ok((
        Lock::load(lock_path(assignments_dir, Pos::Noun))?,
        Lock::load(lock_path(assignments_dir, Pos::Verb))?,
        Lock::load(lock_path(assignments_dir, Pos::Adj))?,
    ))
}

pub fn save_locks(
    assignments_dir: &Path,
    noun_lock: &Lock,
    verb_lock: &Lock,
    adj_lock: &Lock,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(assignments_dir)?;
    noun_lock.save(lock_path(assignments_dir, Pos::Noun))?;
    verb_lock.save(lock_path(assignments_dir, Pos::Verb))?;
    adj_lock.save(lock_path(assignments_dir, Pos::Adj))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn generated_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../english/generated")
    }

    /// The committed tables must survive a parse -> lock -> regenerate round-trip
    /// byte-for-byte. This is the losslessness guard for the bootstrap path; if it
    /// ever fails, the lock no longer faithfully represents the shipped tables.
    #[test]
    fn bootstrap_round_trips_committed_tables_byte_for_byte() {
        let gen_dir = generated_dir();
        let noun_pairs = parse_phf_pairs(gen_dir.join("noun_phf.rs")).unwrap();
        let verb_pairs = parse_phf_pairs(gen_dir.join("verb_phf.rs")).unwrap();
        let adj_pairs = parse_phf_pairs(gen_dir.join("adj_phf.rs")).unwrap();

        let noun_lock = lock_from_pairs(&noun_pairs, Pos::Noun, "test");
        let verb_lock = lock_from_pairs(&verb_pairs, Pos::Verb, "test");
        let adj_lock = lock_from_pairs(&adj_pairs, Pos::Adj, "test");

        let tmp = std::env::temp_dir().join("english_bootstrap_roundtrip");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        generate_tables(&noun_lock, &verb_lock, &adj_lock, &tmp).unwrap();

        for name in ["noun_phf.rs", "verb_phf.rs", "adj_phf.rs"] {
            let committed = fs::read(gen_dir.join(name)).unwrap();
            let regenerated = fs::read(tmp.join(name)).unwrap();
            assert!(
                committed == regenerated,
                "{name} differs after round-trip (lock is not lossless)"
            );
        }
    }
}
