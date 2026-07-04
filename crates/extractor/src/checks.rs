//! Dump-driven accuracy measurement (feature `checks`, which links the `english`
//! crate so the CURRENT committed tables + rule engine are what gets measured).
//!
//! Scoring is per SLOT, not per attested form: Wiktionary frequently attests
//! several valid variants for one slot (`abode` AND `abided`), and a per-form
//! metric could never reach 100% no matter which variant the library picks. A
//! slot counts as a hit when the library's output (through any sense-numbered
//! variant of the lemma) is IN the plain attested set for that slot.
//!
//! Two companion numbers are reported per POS:
//! - the slot accuracy (the headline);
//! - the variant gap: how many plain attested forms the library cannot produce
//!   through any published key (bare or sense-numbered).
//!
//! Soft-tagged (nonstandard/dialectal/...) and excluded (obsolete/...) forms are
//! not counted: by policy the library never emits them. Run
//! `cargo xtask accuracy` before and after any rule/table change so the change
//! carries a number, not an anecdote. Misses are written to per-POS CSVs.
//!
//! Known residual (accepted): adjectives whose periphrastic uses a DIALECTAL
//! degree word ("mair reet"/"maist reet" — Scots) are not admitted as forms, so
//! those slots miss; standard degree words (more/most, farther/further/-est,
//! better/best) are admitted.

#[cfg(feature = "checks")]
use crate::helpers::{Entry, base_setup, entry_is_proper, tag_rank, word_is_proper};
#[cfg(feature = "checks")]
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
#[cfg(feature = "checks")]
use std::io::BufRead;
use std::path::Path;

/// One POS's score: slots reproduced, plus the variant gap.
#[derive(Debug, Clone, Copy, Default)]
pub struct Score {
    /// Slots where the library produced one of the plain attested forms.
    pub matched_slots: u64,
    pub total_slots: u64,
    /// Plain attested forms the library cannot produce for their slot.
    pub unreachable_forms: u64,
}

#[cfg_attr(not(feature = "checks"), allow(dead_code))]
impl Score {
    pub fn percent(&self) -> f64 {
        if self.total_slots == 0 {
            100.0
        } else {
            100.0 * self.matched_slots as f64 / self.total_slots as f64
        }
    }
}

pub fn run_checks(filtered_json_path: &Path, artifacts_dir: &Path) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "checks")]
    {
        let noun = check_noun_plurals(
            filtered_json_path,
            artifacts_dir.join("noun_plural_check.csv"),
        )?;
        let verb =
            check_verb_conjugations(filtered_json_path, artifacts_dir.join("verbs_check.csv"))?;
        let adj = check_adjective_forms(filtered_json_path, artifacts_dir.join("adj_check.csv"))?;

        println!("\naccuracy (slots whose plain attested forms the library reproduces):");
        for (name, s) in [
            ("noun plural", noun),
            ("verb forms", verb),
            ("adj comp/sup", adj),
        ] {
            println!(
                "  {name:>12}: {:>7}/{:<7} = {:.2}%   variant gap: {} attested form(s) not produced",
                s.matched_slots,
                s.total_slots,
                s.percent(),
                s.unreachable_forms,
            );
        }
        Ok(())
    }

    #[cfg(not(feature = "checks"))]
    {
        let _ = filtered_json_path;
        let _ = artifacts_dir;
        Err("extractor was built without the `checks` feature. Re-run with `cargo xtask accuracy` (or `refresh-data --with-checks`).".into())
    }
}

/// The library reaches a homograph sense through its numbered key; a slot counts
/// as reproduced if ANY published key of the lemma produces an attested form.
/// The key list is derived from the ACTUAL generated tables (via
/// `english::__table_entries`), never from a fixed probe window — a hard-coded
/// `_2..=_N` range silently undercounts the moment any lemma's suffixes grow
/// past N.
#[cfg(feature = "checks")]
fn published_keys(table_keys: &[&'static str]) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut by_lemma: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for key in table_keys {
        // The one shared decoder for the emitted-key format (see registry::split_key).
        let lemma = crate::registry::split_key(key)
            .map(|(base, _)| base)
            .unwrap_or(key);
        by_lemma
            .entry(lemma.to_string())
            .or_default()
            .push(key.to_string());
    }
    by_lemma
}

/// The keys to query for one lemma: the bare lemma itself (rule fallback is
/// always reachable) plus every suffixed key the tables actually publish.
#[cfg(feature = "checks")]
fn keys_for(
    by_lemma: &std::collections::BTreeMap<String, Vec<String>>,
    lemma: &str,
) -> Vec<String> {
    let mut keys = vec![lemma.to_string()];
    if let Some(published) = by_lemma.get(lemma) {
        keys.extend(published.iter().filter(|k| *k != lemma).cloned());
    }
    keys
}

/// Score one slot: attested set vs the outputs produced across numbered keys.
#[cfg(feature = "checks")]
fn score_slot(score: &mut Score, attested: &BTreeSet<String>, produced: &BTreeSet<String>) -> bool {
    score.total_slots += 1;
    let hit = attested.iter().any(|f| produced.contains(f));
    if hit {
        score.matched_slots += 1;
    }
    score.unreachable_forms += attested.iter().filter(|f| !produced.contains(*f)).count() as u64;
    hit
}

#[cfg(feature = "checks")]
pub fn check_noun_plurals(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<Score, Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(["lemma", "attested_plurals", "library_said"])?;

    // lemma -> plain attested plurals, aggregated across every entry.
    let mut attested: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry_is_proper(&entry, "noun") {
            continue;
        }
        let lemma = entry.word.to_lowercase();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let f = form.form.to_lowercase();
                if f == "dubious" || !word_is_proper(&f) {
                    continue;
                }
                if form.tags.iter().any(|t| t == "plural") && tag_rank(&form.tags) == Some(0) {
                    attested.entry(lemma.clone()).or_default().insert(f);
                }
            }
        }
    }

    let (noun_keys, _, _) = english::__table_entries();
    let key_strs: Vec<&'static str> = noun_keys.iter().map(|(k, _)| *k).collect();
    let by_lemma = published_keys(&key_strs);

    let mut score = Score::default();
    for (lemma, forms) in &attested {
        let keys = keys_for(&by_lemma, lemma);
        let produced: BTreeSet<String> = keys
            .iter()
            .map(|v| English::noun(v, &Number::Plural))
            .collect();
        if !score_slot(&mut score, forms, &produced) {
            let shown: Vec<&str> = forms.iter().map(|s| s.as_str()).collect();
            writer.write_record(&[
                lemma.clone(),
                shown.join("|"),
                English::noun(lemma, &Number::Plural),
            ])?;
        }
    }

    writer.flush()?;
    println!(
        "noun plural check: {}/{} slots ({:.2}%); misses in {}",
        score.matched_slots,
        score.total_slots,
        score.percent(),
        output_path.display()
    );
    Ok(score)
}

#[cfg(feature = "checks")]
pub fn check_verb_conjugations(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<Score, Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(["lemma", "slot", "attested_forms", "library_said"])?;

    const SLOTS: usize = 4; // 3sg, past, pres_part, past_part
    let slot_name = ["3sg", "past", "pres_part", "past_part"];
    let mut attested: BTreeMap<String, [BTreeSet<String>; SLOTS]> = BTreeMap::new();

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry_is_proper(&entry, "verb") {
            continue;
        }
        let lemma = entry.word.to_lowercase();
        if lemma == "be" {
            continue; // handled directly by english-core, not the tables
        }
        if let Some(forms) = &entry.forms {
            for form in forms {
                let f = form.form.to_lowercase();
                if f == "dubious" || !word_is_proper(&f) || tag_rank(&form.tags) != Some(0) {
                    continue;
                }
                let has = |t: &str| form.tags.iter().any(|x| x == t);
                let slot = if has("third-person") && has("singular") && has("present") {
                    0
                } else if has("participle") && has("present") {
                    2
                } else if has("participle") && has("past") {
                    3
                } else if has("past") {
                    1
                } else {
                    continue;
                };
                attested.entry(lemma.clone()).or_default()[slot].insert(f);
            }
        }
    }

    let slot_query = |v: &str, slot: usize| match slot {
        0 => English::verb(
            v,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
        ),
        1 => English::verb(
            v,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite,
        ),
        2 => English::verb(
            v,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle,
        ),
        _ => English::verb(
            v,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Participle,
        ),
    };

    let (_, verb_keys, _) = english::__table_entries();
    let key_strs: Vec<&'static str> = verb_keys.iter().map(|(k, _)| *k).collect();
    let by_lemma = published_keys(&key_strs);

    let mut score = Score::default();
    for (lemma, slots) in &attested {
        let keys = keys_for(&by_lemma, lemma);
        for (slot, forms) in slots.iter().enumerate() {
            if forms.is_empty() {
                continue;
            }
            let produced: BTreeSet<String> = keys.iter().map(|v| slot_query(v, slot)).collect();
            if !score_slot(&mut score, forms, &produced) {
                let shown: Vec<&str> = forms.iter().map(|s| s.as_str()).collect();
                writer.write_record(&[
                    lemma.clone(),
                    slot_name[slot].to_string(),
                    shown.join("|"),
                    slot_query(lemma, slot),
                ])?;
            }
        }
    }

    writer.flush()?;
    println!(
        "verb form check: {}/{} slots ({:.2}%); misses in {}",
        score.matched_slots,
        score.total_slots,
        score.percent(),
        output_path.display()
    );
    Ok(score)
}

#[cfg(feature = "checks")]
pub fn check_adjective_forms(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<Score, Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(["lemma", "degree", "attested_forms", "library_said"])?;

    // Periphrastic attestations ("more free") are legitimate for adjectives, so
    // no word_is_proper filter on the forms here.
    let mut attested: BTreeMap<String, [BTreeSet<String>; 2]> = BTreeMap::new();
    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry_is_proper(&entry, "adj") {
            continue;
        }
        let lemma = entry.word.to_lowercase();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let f = form.form.to_lowercase();
                if f == "dubious" || tag_rank(&form.tags) != Some(0) {
                    continue;
                }
                if form.tags.iter().any(|t| t == "comparative") {
                    attested.entry(lemma.clone()).or_default()[0].insert(f.clone());
                }
                if form.tags.iter().any(|t| t == "superlative") {
                    attested.entry(lemma.clone()).or_default()[1].insert(f);
                }
            }
        }
    }

    let (_, _, adj_keys) = english::__table_entries();
    let key_strs: Vec<&'static str> = adj_keys.iter().map(|(k, _)| *k).collect();
    let by_lemma = published_keys(&key_strs);

    let mut score = Score::default();
    for (lemma, slots) in &attested {
        let keys = keys_for(&by_lemma, lemma);
        for (slot, forms) in slots.iter().enumerate() {
            if forms.is_empty() {
                continue;
            }
            let degree = if slot == 0 {
                Degree::Comparative
            } else {
                Degree::Superlative
            };
            let produced: BTreeSet<String> =
                keys.iter().map(|v| English::adj(v, &degree)).collect();
            if !score_slot(&mut score, forms, &produced) {
                let shown: Vec<&str> = forms.iter().map(|s| s.as_str()).collect();
                writer.write_record(&[
                    lemma.clone(),
                    if slot == 0 {
                        "Comparative"
                    } else {
                        "Superlative"
                    }
                    .to_string(),
                    shown.join("|"),
                    English::adj(lemma, &degree),
                ])?;
            }
        }
    }

    writer.flush()?;
    println!(
        "adj form check: {}/{} slots ({:.2}%); misses in {}",
        score.matched_slots,
        score.total_slots,
        score.percent(),
        output_path.display()
    );
    Ok(score)
}
