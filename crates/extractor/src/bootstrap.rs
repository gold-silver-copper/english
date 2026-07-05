//! Table emission and parsing.
//!
//! [`generate_tables`] is the single candidate -> PHF path, byte-deterministic
//! because `check-registry` and the accuracy harness both read the committed
//! tables back with [`parse_phf_pairs`]. There is no lockfile: the generated
//! tables ARE the committed artifact.

use crate::assign::split_key;
use crate::extract::Table;
use crate::file_generation::{
    write_adjectives_phf, write_adverbs_phf, write_nouns_phf, write_verbs_phf,
};
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

/// Emit the noun/verb/adj/adverb PHF tables from the extracted `(key, forms)`
/// rows. The single source-of-truth -> tables path; a duplicate key is refused
/// here rather than surfacing as an opaque `phf_map!` compile error downstream.
pub fn generate_tables(
    noun: &Table,
    verb: &Table,
    adj: &Table,
    adverb: &Table,
    generated_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    reject_duplicate_keys(noun, "noun")?;
    reject_duplicate_keys(verb, "verb")?;
    reject_duplicate_keys(adj, "adj")?;
    reject_duplicate_keys(adverb, "adverb")?;

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

    // Adjectives: two columns (comparative, superlative).
    let adj_pairs: Vec<(String, (String, String))> = adj
        .iter()
        .map(|(k, f)| (k.clone(), (col(f, 0), col(f, 1))))
        .collect();
    write_adjectives_phf(adj_pairs, generated_dir.join("adj_phf.rs"))?;

    // Adverbs: two columns (comparative, superlative), same arity as adjectives.
    let adverb_pairs: Vec<(String, (String, String))> = adverb
        .iter()
        .map(|(k, f)| (k.clone(), (col(f, 0), col(f, 1))))
        .collect();
    write_adverbs_phf(adverb_pairs, generated_dir.join("adverb_phf.rs"))?;

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
///   core rule changed without regenerating).
///
/// It does NOT verify that a row's irregular VALUES are correct/current — those
/// are attested data, not derivable without the dump; `cargo xtask accuracy` is
/// the authoritative value check. Returns human-readable violations; empty = OK.
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
    audit_pos(&mut v, generated_dir, "adverb_phf.rs", 2, |lemma, forms| {
        forms[0] == EnglishCore::comparative_adverb(lemma)
            && forms[1] == EnglishCore::superlative_adverb(lemma)
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
    audit_rows(v, name, arity, rule_predicts, &pairs);
    Ok(())
}

/// The per-row structural + layering checks, separated from file I/O so they are
/// unit-testable with in-memory rows.
fn audit_rows(
    v: &mut Vec<String>,
    name: &str,
    arity: usize,
    rule_predicts: impl Fn(&str, &[String]) -> bool,
    pairs: &[KeyForms],
) {
    let mut seen: HashSet<&str> = HashSet::new();
    for (key, forms) in pairs {
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
            // A BARE key must hold something the rules cannot predict. The generator
            // never emits a rule-equal bare key (a rule-equal candidate is dropped
            // and the bare slot reserved for the rule; a soft one sorts after its
            // standard sibling), so any bare row that DOES equal the rule is dead
            // weight — a core rule changed without regenerating. No sibling
            // exemption: it would only blind this check for homograph bases.
            None => {
                if rule_predicts(key, forms) {
                    v.push(format!(
                        "{name}: bare key `{key}` equals the regular-rule prediction — dead weight; \
                         a core rule changed without regenerating (`cargo xtask refresh-data`)"
                    ));
                }
            }
        }
    }
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
        let adverb = parse_phf_pairs(gen_dir.join("adverb_phf.rs")).unwrap();

        let tmp =
            std::env::temp_dir().join(format!("english_tables_roundtrip_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        generate_tables(&noun, &verb, &adj, &adverb, &tmp).unwrap();

        for name in ["noun_phf.rs", "verb_phf.rs", "adj_phf.rs", "adverb_phf.rs"] {
            let committed = fs::read(gen_dir.join(name)).unwrap();
            let regenerated = fs::read(tmp.join(name)).unwrap();
            assert!(
                committed == regenerated,
                "{name} differs after a parse -> emit round-trip"
            );
        }
    }

    fn rows(v: &[(&str, &[&str])]) -> Vec<KeyForms> {
        v.iter()
            .map(|(k, f)| (k.to_string(), f.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn audit_rows_flags_structural_and_layering_problems() {
        // A fake "add -s" noun rule for the layering check.
        let adds_s = |lemma: &str, forms: &[String]| forms == [format!("{lemma}s")];

        // Clean table: a bare irregular (child != childs) plus a numbered variant.
        let mut out = Vec::new();
        audit_rows(
            &mut out,
            "noun",
            1,
            adds_s,
            &rows(&[("child", &["children"]), ("child_2", &["childz"])]),
        );
        assert!(out.is_empty(), "clean table flagged: {out:?}");

        // A rule-equal bare row is dead weight — flagged EVEN with a numbered
        // sibling (the exemption that used to hide this is gone).
        let mut out = Vec::new();
        audit_rows(
            &mut out,
            "noun",
            1,
            adds_s,
            &rows(&[("dog", &["dogs"]), ("dog_2", &["doggos"])]),
        );
        assert!(
            out.iter()
                .any(|m| m.contains("bare key `dog`") && m.contains("dead weight")),
            "{out:?}"
        );

        // Duplicate key, wrong arity, empty column, and a suffix < 2.
        let never = |_: &str, _: &[String]| false;
        let mut out = Vec::new();
        audit_rows(
            &mut out,
            "noun",
            1,
            never,
            &rows(&[("x", &["xen"]), ("x", &["xim"])]),
        );
        assert!(
            out.iter().any(|m| m.contains("duplicate key `x`")),
            "{out:?}"
        );
        let mut out = Vec::new();
        audit_rows(&mut out, "verb", 4, never, &rows(&[("v", &["a", "b"])]));
        assert!(
            out.iter().any(|m| m.contains("expected 4")),
            "arity: {out:?}"
        );
        let mut out = Vec::new();
        audit_rows(&mut out, "noun", 1, never, &rows(&[("e", &[""])]));
        assert!(
            out.iter().any(|m| m.contains("empty form column")),
            "{out:?}"
        );
        let mut out = Vec::new();
        audit_rows(&mut out, "noun", 1, never, &rows(&[("z_1", &["zs"])]));
        assert!(out.iter().any(|m| m.contains("suffix 1")), "{out:?}");
    }

    #[test]
    fn parse_phf_pairs_reads_periphrastic_forms_with_spaces() {
        // Periphrastic adjective forms ("more free") contain spaces; the '"'-split
        // parser must recover them intact.
        let dir =
            std::env::temp_dir().join(format!("english_parse_phf_spaces_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("adj_phf.rs");
        fs::write(
            &path,
            "use phf::phf_map;\n\
             pub static ADJECTIVE_MAP: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {\n    \
             \"free_2\" => (\"more free\", \"most free\"),\n};\n",
        )
        .unwrap();
        let pairs = parse_phf_pairs(&path).unwrap();
        assert_eq!(
            pairs,
            vec![(
                "free_2".to_string(),
                vec!["more free".to_string(), "most free".to_string()]
            )]
        );
    }
}
