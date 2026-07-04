//! Table emission and parsing.
//!
//! [`generate_tables`] is the single candidate -> PHF path, byte-deterministic
//! because `check-registry` and the accuracy harness both read the committed
//! tables back with [`parse_phf_pairs`]. There is no lockfile: the generated
//! tables ARE the committed artifact.

use crate::assign::split_key;
use crate::extract::Table;
use crate::file_generation::{write_adjectives_phf, write_nouns_phf, write_verbs_phf};
use english_core::EnglishCore;
use english_core::grammar::{Form, Number, Person, Tense};
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

/// A parsed table row: a key and its inflected form columns.
pub type KeyForms = (String, Vec<String>);

/// Extract `(key, form-columns)` pairs from a generated `*_phf.rs` file by reading
/// the quoted string literals on each `"key" => ...,` line. The first quoted
/// field is the key; the rest are the inflected forms in column order. This is the
/// emitter-independent reader `check-registry` and the accuracy harness use.
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

/// Emit the noun/verb/adj PHF tables from the extracted `(key, forms)` rows. The
/// single source-of-truth -> tables path; a duplicate key is refused here rather
/// than surfacing as an opaque `phf_map!` compile error downstream.
pub fn generate_tables(
    noun: &Table,
    verb: &Table,
    adj: &Table,
    generated_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    reject_duplicate_keys(noun, "noun")?;
    reject_duplicate_keys(verb, "verb")?;
    reject_duplicate_keys(adj, "adj")?;

    // Nouns: one plural column.
    let noun_pairs: Vec<(String, String)> =
        noun.iter().map(|(k, f)| (k.clone(), col(f, 0))).collect();
    write_nouns_phf(noun_pairs, generated_dir.join("noun_phf.rs"))?;

    // Verbs: four columns.
    let verb_pairs: Vec<(String, (String, String, String, String))> = verb
        .iter()
        .map(|(k, f)| (k.clone(), (col(f, 0), col(f, 1), col(f, 2), col(f, 3))))
        .collect();
    write_verbs_phf(verb_pairs, generated_dir.join("verb_phf.rs"))?;

    // Adjectives: two columns.
    let adj_pairs: Vec<(String, (String, String))> = adj
        .iter()
        .map(|(k, f)| (k.clone(), (col(f, 0), col(f, 1))))
        .collect();
    write_adjectives_phf(adj_pairs, generated_dir.join("adj_phf.rs"))?;

    Ok(())
}

fn reject_duplicate_keys(table: &Table, pos: &str) -> Result<(), Box<dyn Error>> {
    let mut seen: HashSet<&str> = HashSet::new();
    let dups: Vec<&str> = table
        .iter()
        .filter_map(|(k, _)| (!seen.insert(k)).then_some(k.as_str()))
        .collect();
    if dups.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "refusing to generate {pos} table — duplicate key(s): {}",
            dups.join(", ")
        )
        .into())
    }
}

fn col(forms: &[String], i: usize) -> String {
    forms.get(i).cloned().unwrap_or_default()
}

/// The dump-free consistency gate behind `cargo xtask check-registry`. Reads the
/// committed PHF tables back with [`parse_phf_pairs`] (an emitter-independent
/// path) and verifies, WITHOUT any lockfile or dump:
/// - keys are well-formed and unique (a numbered key is `lemma_<n>` with `n >= 2`);
/// - every row has the right column arity with no empty column;
/// - rule/table layering holds — a BARE key must hold something the regular rules
///   cannot predict (a bare row equal to the rule output is dead weight and means a
///   core rule changed without regenerating). A bare key that has a numbered
///   sibling is exempt: it is the base of a homograph set the runtime falls back to.
///
/// Returns human-readable violations; empty = OK.
pub fn audit_tables(generated_dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut v = Vec::new();
    audit_pos(&mut v, generated_dir, "noun_phf.rs", 1, |lemma, forms| {
        forms[0] == EnglishCore::pluralize_noun(lemma)
    })?;
    audit_pos(&mut v, generated_dir, "verb_phf.rs", 4, |lemma, forms| {
        let p =
            |t: &Tense, f: &Form| EnglishCore::verb(lemma, &Person::Third, &Number::Singular, t, f);
        forms[0] == p(&Tense::Present, &Form::Finite)
            && forms[1] == p(&Tense::Past, &Form::Finite)
            && forms[2] == p(&Tense::Present, &Form::Participle)
            && forms[3] == p(&Tense::Past, &Form::Finite)
    })?;
    audit_pos(&mut v, generated_dir, "adj_phf.rs", 2, |lemma, forms| {
        forms[0] == EnglishCore::comparative(lemma) && forms[1] == EnglishCore::superlative(lemma)
    })?;
    Ok(v)
}

fn audit_pos(
    v: &mut Vec<String>,
    generated_dir: &Path,
    name: &str,
    arity: usize,
    rule_predicts: impl Fn(&str, &[String]) -> bool,
) -> Result<(), Box<dyn Error>> {
    let pairs = parse_phf_pairs(generated_dir.join(name))?;
    if pairs.is_empty() {
        v.push(format!(
            "{name}: no table entries parsed (missing or malformed table?)"
        ));
        return Ok(());
    }
    // Lemmas that carry a numbered key: a bare row for one of these is the base of
    // a homograph set and is exempt from the dead-weight-vs-rule check.
    let suffixed_lemmas: HashSet<&str> = pairs
        .iter()
        .filter_map(|(k, _)| split_key(k).map(|(lemma, _)| lemma))
        .collect();
    let mut seen: HashSet<&str> = HashSet::new();
    for (key, forms) in &pairs {
        if !seen.insert(key) {
            v.push(format!("{name}: duplicate key `{key}`"));
        }
        if forms.len() != arity {
            v.push(format!(
                "{name}: key `{key}` has {} form column(s), expected {arity}",
                forms.len()
            ));
            continue;
        }
        if forms.iter().any(|f| f.is_empty()) {
            v.push(format!("{name}: key `{key}` has an empty form column"));
        }
        match split_key(key) {
            Some((_, suffix)) if suffix < 2 => v.push(format!(
                "{name}: numbered key `{key}` has suffix {suffix} (numbered keys start at 2)"
            )),
            Some(_) => {}
            None => {
                if !suffixed_lemmas.contains(key.as_str()) && rule_predicts(key, forms) {
                    v.push(format!(
                        "{name}: bare key `{key}` equals the regular-rule prediction — dead weight; \
                         a core rule changed without regenerating (`cargo xtask refresh-data`)"
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn generated_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../english/generated")
    }

    /// The emitter is a stable round-trip: parsing the committed tables and
    /// re-emitting them must reproduce them byte-for-byte. If this fails, the
    /// `check-registry` regeneration would spuriously diff against the committed
    /// tables (or the emitter formatting drifted).
    #[test]
    fn tables_round_trip_committed_output_byte_for_byte() {
        let gen_dir = generated_dir();
        let noun = parse_phf_pairs(gen_dir.join("noun_phf.rs")).unwrap();
        let verb = parse_phf_pairs(gen_dir.join("verb_phf.rs")).unwrap();
        let adj = parse_phf_pairs(gen_dir.join("adj_phf.rs")).unwrap();

        let tmp = std::env::temp_dir().join("english_tables_roundtrip");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        generate_tables(&noun, &verb, &adj, &tmp).unwrap();

        for name in ["noun_phf.rs", "verb_phf.rs", "adj_phf.rs"] {
            let committed = fs::read(gen_dir.join(name)).unwrap();
            let regenerated = fs::read(tmp.join(name)).unwrap();
            assert!(
                committed == regenerated,
                "{name} differs after a parse -> emit round-trip"
            );
        }
    }
}
