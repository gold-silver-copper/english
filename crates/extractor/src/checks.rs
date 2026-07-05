//! Dump-driven accuracy measurement (feature `checks`, which links the `english`
//! crate so the CURRENT committed tables + rule engine are what gets measured).
//!
//! Scoring is per SLOT, not per attested form: Wiktionary frequently attests
//! several valid variants for one slot (`abode` AND `abided`), and a per-form
//! metric could never reach 100% no matter which variant the library picks. A
//! slot counts as a hit when the library's output (through any sense-numbered
//! variant of the lemma) is IN the plain attested set for that slot.
//!
//! Three companion numbers are reported per POS:
//! - the slot accuracy (the headline);
//! - the variant gap: how many plain attested forms the library cannot produce
//!   through any published key (bare or sense-numbered);
//! - bare-lemma correctness ([`BareScore`]): the slot metric unions every key,
//!   so it is blind to a standard form demoted to an `_n` key while the bare
//!   lemma returns a non-standard one (`album` -> `alba`, standard `albums` at
//!   `album_2`). This measures what `English::<pos>(lemma)` — the natural call —
//!   returns against the PRIMARY (first-listed, most-standard) attested form, and
//!   separately counts over-generation (an inflected form for an invariant slot).
//!   Both are invisible to the slot metric, and both stay at "100%" today.
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

    /// The percentage as a display string that never rounds a NON-perfect score
    /// up to `100.00%` — an imperfect metric shown as `100%` is misleading
    /// (`121548/121550` is `99.998%`, not `100.00%`). A perfect score prints
    /// `100.00%`; an imperfect one uses the fewest decimals (>= 2) that still
    /// render strictly below 100.
    pub fn percent_display(&self) -> String {
        if self.total_slots == 0 || self.matched_slots >= self.total_slots {
            return "100.00%".to_string();
        }
        let pct = self.percent();
        for prec in 2..=6usize {
            let factor = 10f64.powi(prec as i32);
            if (pct * factor).round() / factor < 100.0 {
                return format!("{pct:.prec$}%");
            }
        }
        // Astronomically close to 100 but not equal: floor so it never shows 100.
        format!("{:.2}%", (pct * 100.0).floor() / 100.0)
    }
}

/// Bare-lemma correctness — a companion to [`Score`] that measures what the
/// NATURAL call returns, not what is merely reachable.
///
/// [`Score`] unions every published key (`keys_for`), so it reports a hit
/// whenever ANY key of the lemma reproduces an attested form. That is blind to a
/// standard form demoted to an `_n` sense key while the bare lemma yields a
/// non-standard one — `English::noun("album")` returns the rule's `alba` while
/// the standard `albums` sits at `album_2`; the slot still "hits" and the
/// headline stays 100%. This companion scores the BARE lemma alone against the
/// PRIMARY (first-listed, i.e. most-standard) attested form, and separately
/// counts over-generation: an inflected form for a slot the dump attests as
/// invariant. Both are invisible to the slot metric by construction.
#[derive(Debug, Clone, Copy, Default)]
pub struct BareScore {
    /// Slots where the bare lemma alone reproduced the primary attested form.
    pub bare_primary_hits: u64,
    /// Slots scored (same denominator as the slot metric).
    pub total: u64,
    /// Slots that the slot metric counts as a hit, yet the bare lemma does not
    /// return the primary form even though the primary IS produced by some other
    /// published key — i.e. the standard form was demoted to an `_n` key.
    pub demoted: u64,
    /// Slots the dump attests as invariant (primary == base) where the library
    /// nonetheless emits a changed, inflected form — over-generation.
    pub overgenerated: u64,
}

/// Does the bare-lemma output reproduce the primary (first-listed, i.e.
/// most-standard) attested form for the slot?
#[cfg(feature = "checks")]
fn bare_is_primary(bare: &str, primary: Option<&str>) -> bool {
    primary == Some(bare)
}

/// Over-generation: the dump attests the slot as invariant (its primary form is
/// the unchanged base), yet the library emitted a different, inflected form.
#[cfg(feature = "checks")]
fn is_overgeneration(bare: &str, base: &str, primary: Option<&str>) -> bool {
    primary == Some(base) && bare != base
}

/// Fold one slot into a [`BareScore`], returning whether the bare lemma produced
/// the primary form. `primary_reachable` says whether the primary form is
/// produced by SOME published key — so a non-primary bare output is a demotion
/// (the standard form lives at an `_n` key) rather than simply unreachable.
///
/// The buckets are mutually exclusive and match the per-row CSV categorization:
/// a non-primary slot is over-generation (invariant slot the library inflected),
/// else a demotion (standard form reachable via another key), else an
/// uncategorized bare miss. Over-generation takes priority so an invariant that
/// happens to be reachable is not also counted as a demotion.
#[cfg(feature = "checks")]
fn score_bare(
    bs: &mut BareScore,
    bare: &str,
    base: &str,
    primary: Option<&str>,
    union_hit: bool,
    primary_reachable: bool,
) -> bool {
    bs.total += 1;
    let is_primary = bare_is_primary(bare, primary);
    if is_primary {
        bs.bare_primary_hits += 1;
    } else if is_overgeneration(bare, base, primary) {
        bs.overgenerated += 1;
    } else if union_hit && primary.is_some() && primary_reachable {
        bs.demoted += 1;
    }
    is_primary
}

pub fn run_checks(filtered_json_path: &Path, artifacts_dir: &Path) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "checks")]
    {
        let (noun, noun_bare) = check_noun_plurals(
            filtered_json_path,
            artifacts_dir.join("noun_plural_check.csv"),
            artifacts_dir.join("noun_bare_check.csv"),
        )?;
        let (verb, verb_bare) = check_verb_conjugations(
            filtered_json_path,
            artifacts_dir.join("verbs_check.csv"),
            artifacts_dir.join("verbs_bare_check.csv"),
        )?;
        let (adj, adj_bare) = check_adjective_forms(
            filtered_json_path,
            artifacts_dir.join("adj_check.csv"),
            artifacts_dir.join("adj_bare_check.csv"),
        )?;
        let (adverb, adverb_bare) = check_adverb_forms(
            filtered_json_path,
            artifacts_dir.join("adverb_check.csv"),
            artifacts_dir.join("adverb_bare_check.csv"),
        )?;

        println!(
            "\naccuracy (slots whose plain attested forms the library reproduces through ANY published key):"
        );
        for (name, s) in [
            ("noun plural", noun),
            ("verb forms", verb),
            ("adj comp/sup", adj),
            ("adverb comp/sup", adverb),
        ] {
            println!(
                "  {name:>15}: {:>7}/{:<7} = {}   variant gap: {} attested form(s) not produced",
                s.matched_slots,
                s.total_slots,
                s.percent_display(),
                s.unreachable_forms,
            );
        }

        println!(
            "\nbare-lemma correctness (does English::<pos>(lemma) alone return the primary — most-standard, first-listed — attested form?):"
        );
        for (name, b) in [
            ("noun plural", noun_bare),
            ("verb forms", verb_bare),
            ("adj comp/sup", adj_bare),
            ("adverb comp/sup", adverb_bare),
        ] {
            // Reuse the tested never-round-up-to-100 formatter by borrowing Score's.
            let pct = Score {
                matched_slots: b.bare_primary_hits,
                total_slots: b.total,
                unreachable_forms: 0,
            }
            .percent_display();
            println!(
                "  {name:>15}: {:>7}/{:<7} = {}   standard-form demoted to _n key: {}   over-generated: {}",
                b.bare_primary_hits, b.total, pct, b.demoted, b.overgenerated,
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
/// The key list is derived from the ACTUAL generated tables, never from a fixed
/// probe window — a hard-coded `_2..=_N` range silently undercounts the moment any
/// lemma's suffixes grow past N.
#[cfg(feature = "checks")]
fn generated_keys(name: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../english/generated")
        .join(name);
    Ok(crate::bootstrap::parse_phf_pairs(path)?
        .into_iter()
        .map(|(k, _)| k)
        .collect())
}

#[cfg(feature = "checks")]
fn published_keys(table_keys: &[String]) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut by_lemma: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for key in table_keys {
        let key = key.as_str();
        // The one shared decoder for the emitted-key format (see assign::split_key).
        let lemma = crate::assign::split_key(key)
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
    bare_output_path: impl AsRef<Path>,
) -> Result<(Score, BareScore), Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(["lemma", "attested_plurals", "library_said"])?;
    let mut bare_writer = csv::Writer::from_path(bare_output_path.as_ref())?;
    bare_writer.write_record(["lemma", "primary_attested", "bare_said", "category"])?;

    // lemma -> plain attested plurals, aggregated across every entry; plus the
    // first-listed plural per lemma (its "primary", most-standard form).
    let mut attested: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut primary: BTreeMap<String, String> = BTreeMap::new();
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
                    primary.entry(lemma.clone()).or_insert_with(|| f.clone());
                    attested.entry(lemma.clone()).or_default().insert(f);
                }
            }
        }
    }

    let noun_keys = generated_keys("noun_phf.rs")?;
    let by_lemma = published_keys(&noun_keys);

    let mut score = Score::default();
    let mut bare = BareScore::default();
    for (lemma, forms) in &attested {
        let keys = keys_for(&by_lemma, lemma);
        let produced: BTreeSet<String> = keys
            .iter()
            .map(|v| English::noun(v, &Number::Plural))
            .collect();
        let bare_said = English::noun(lemma, &Number::Plural);
        let union_hit = score_slot(&mut score, forms, &produced);
        if !union_hit {
            let shown: Vec<&str> = forms.iter().map(|s| s.as_str()).collect();
            writer.write_record(&[lemma.clone(), shown.join("|"), bare_said.clone()])?;
        }

        let prim = primary.get(lemma).map(String::as_str);
        let primary_reachable = prim.is_some_and(|p| produced.contains(p));
        if !score_bare(
            &mut bare,
            &bare_said,
            lemma,
            prim,
            union_hit,
            primary_reachable,
        ) {
            let category = if is_overgeneration(&bare_said, lemma, prim) {
                "over-generation"
            } else if union_hit && primary_reachable {
                "standard-demoted-to-_n"
            } else {
                "bare-miss"
            };
            bare_writer.write_record(&[
                lemma.clone(),
                prim.unwrap_or("").to_string(),
                bare_said,
                category.to_string(),
            ])?;
        }
    }

    writer.flush()?;
    bare_writer.flush()?;
    println!(
        "noun plural check: {}/{} slots ({}); bare-form primary {}/{} (demoted {}, over-gen {}); misses in {}",
        score.matched_slots,
        score.total_slots,
        score.percent_display(),
        bare.bare_primary_hits,
        bare.total,
        bare.demoted,
        bare.overgenerated,
        output_path.display()
    );
    Ok((score, bare))
}

#[cfg(feature = "checks")]
pub fn check_verb_conjugations(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    bare_output_path: impl AsRef<Path>,
) -> Result<(Score, BareScore), Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(["lemma", "slot", "attested_forms", "library_said"])?;
    let mut bare_writer = csv::Writer::from_path(bare_output_path.as_ref())?;
    bare_writer.write_record(["lemma", "slot", "primary_attested", "bare_said", "category"])?;

    const SLOTS: usize = 4; // 3sg, past, pres_part, past_part
    let slot_name = ["3sg", "past", "pres_part", "past_part"];
    let mut attested: BTreeMap<String, [BTreeSet<String>; SLOTS]> = BTreeMap::new();
    // First-listed attested form per (lemma, slot): the primary/most-standard one.
    let mut primary: BTreeMap<String, [Option<String>; SLOTS]> = BTreeMap::new();

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
        // `be` is served by english-core's suppletive path rather than the tables,
        // but the library still produces its forms, so it IS scored here (the 3sg
        // slots am/is/are collapse to `is`/`was`/`being`/`been` at 3rd-singular).
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
                primary.entry(lemma.clone()).or_default()[slot].get_or_insert_with(|| f.clone());
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

    let verb_keys = generated_keys("verb_phf.rs")?;
    let by_lemma = published_keys(&verb_keys);

    let mut score = Score::default();
    let mut bare = BareScore::default();
    for (lemma, slots) in &attested {
        let keys = keys_for(&by_lemma, lemma);
        let prims = primary.get(lemma);
        for (slot, forms) in slots.iter().enumerate() {
            if forms.is_empty() {
                continue;
            }
            let produced: BTreeSet<String> = keys.iter().map(|v| slot_query(v, slot)).collect();
            let bare_said = slot_query(lemma, slot);
            let union_hit = score_slot(&mut score, forms, &produced);
            if !union_hit {
                let shown: Vec<&str> = forms.iter().map(|s| s.as_str()).collect();
                writer.write_record(&[
                    lemma.clone(),
                    slot_name[slot].to_string(),
                    shown.join("|"),
                    bare_said.clone(),
                ])?;
            }

            let prim = prims.and_then(|p| p[slot].as_deref());
            let primary_reachable = prim.is_some_and(|p| produced.contains(p));
            if !score_bare(
                &mut bare,
                &bare_said,
                lemma,
                prim,
                union_hit,
                primary_reachable,
            ) {
                let category = if is_overgeneration(&bare_said, lemma, prim) {
                    "over-generation"
                } else if union_hit && primary_reachable {
                    "standard-demoted-to-_n"
                } else {
                    "bare-miss"
                };
                bare_writer.write_record(&[
                    lemma.clone(),
                    slot_name[slot].to_string(),
                    prim.unwrap_or("").to_string(),
                    bare_said,
                    category.to_string(),
                ])?;
            }
        }
    }

    writer.flush()?;
    bare_writer.flush()?;
    println!(
        "verb form check: {}/{} slots ({}); bare-form primary {}/{} (demoted {}, over-gen {}); misses in {}",
        score.matched_slots,
        score.total_slots,
        score.percent_display(),
        bare.bare_primary_hits,
        bare.total,
        bare.demoted,
        bare.overgenerated,
        output_path.display()
    );
    Ok((score, bare))
}

#[cfg(feature = "checks")]
pub fn check_adjective_forms(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    bare_output_path: impl AsRef<Path>,
) -> Result<(Score, BareScore), Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(["lemma", "degree", "attested_forms", "library_said"])?;
    let mut bare_writer = csv::Writer::from_path(bare_output_path.as_ref())?;
    bare_writer.write_record([
        "lemma",
        "degree",
        "primary_attested",
        "bare_said",
        "category",
    ])?;

    // Periphrastic attestations ("more free") are legitimate for adjectives, so
    // no word_is_proper filter on the forms here.
    let mut attested: BTreeMap<String, [BTreeSet<String>; 2]> = BTreeMap::new();
    let mut primary: BTreeMap<String, [Option<String>; 2]> = BTreeMap::new();
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
                    primary.entry(lemma.clone()).or_default()[0].get_or_insert_with(|| f.clone());
                    attested.entry(lemma.clone()).or_default()[0].insert(f.clone());
                }
                if form.tags.iter().any(|t| t == "superlative") {
                    primary.entry(lemma.clone()).or_default()[1].get_or_insert_with(|| f.clone());
                    attested.entry(lemma.clone()).or_default()[1].insert(f);
                }
            }
        }
    }

    let adj_keys = generated_keys("adj_phf.rs")?;
    let by_lemma = published_keys(&adj_keys);

    let mut score = Score::default();
    let mut bare = BareScore::default();
    for (lemma, slots) in &attested {
        let keys = keys_for(&by_lemma, lemma);
        let prims = primary.get(lemma);
        for (slot, forms) in slots.iter().enumerate() {
            if forms.is_empty() {
                continue;
            }
            let degree = if slot == 0 {
                Degree::Comparative
            } else {
                Degree::Superlative
            };
            let label = if slot == 0 {
                "Comparative"
            } else {
                "Superlative"
            };
            let produced: BTreeSet<String> =
                keys.iter().map(|v| English::adj(v, &degree)).collect();
            let bare_said = English::adj(lemma, &degree);
            let union_hit = score_slot(&mut score, forms, &produced);
            if !union_hit {
                let shown: Vec<&str> = forms.iter().map(|s| s.as_str()).collect();
                writer.write_record(&[
                    lemma.clone(),
                    label.to_string(),
                    shown.join("|"),
                    bare_said.clone(),
                ])?;
            }

            let prim = prims.and_then(|p| p[slot].as_deref());
            let primary_reachable = prim.is_some_and(|p| produced.contains(p));
            if !score_bare(
                &mut bare,
                &bare_said,
                lemma,
                prim,
                union_hit,
                primary_reachable,
            ) {
                let category = if union_hit && primary_reachable {
                    "standard-demoted-to-_n"
                } else {
                    "bare-miss"
                };
                bare_writer.write_record(&[
                    lemma.clone(),
                    label.to_string(),
                    prim.unwrap_or("").to_string(),
                    bare_said,
                    category.to_string(),
                ])?;
            }
        }
    }

    writer.flush()?;
    bare_writer.flush()?;
    println!(
        "adj form check: {}/{} slots ({}); bare-form primary {}/{} (demoted {}); misses in {}",
        score.matched_slots,
        score.total_slots,
        score.percent_display(),
        bare.bare_primary_hits,
        bare.total,
        bare.demoted,
        output_path.display()
    );
    Ok((score, bare))
}

#[cfg(feature = "checks")]
pub fn check_adverb_forms(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    bare_output_path: impl AsRef<Path>,
) -> Result<(Score, BareScore), Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(["lemma", "degree", "attested_forms", "library_said"])?;
    let mut bare_writer = csv::Writer::from_path(bare_output_path.as_ref())?;
    bare_writer.write_record([
        "lemma",
        "degree",
        "primary_attested",
        "bare_said",
        "category",
    ])?;

    // Periphrastic attestations ("more quickly") are the norm for adverbs, so no
    // word_is_proper filter on the forms here.
    let mut attested: BTreeMap<String, [BTreeSet<String>; 2]> = BTreeMap::new();
    let mut primary: BTreeMap<String, [Option<String>; 2]> = BTreeMap::new();
    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry_is_proper(&entry, "adv") {
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
                    primary.entry(lemma.clone()).or_default()[0].get_or_insert_with(|| f.clone());
                    attested.entry(lemma.clone()).or_default()[0].insert(f.clone());
                }
                if form.tags.iter().any(|t| t == "superlative") {
                    primary.entry(lemma.clone()).or_default()[1].get_or_insert_with(|| f.clone());
                    attested.entry(lemma.clone()).or_default()[1].insert(f);
                }
            }
        }
    }

    let adverb_keys = generated_keys("adverb_phf.rs")?;
    let by_lemma = published_keys(&adverb_keys);

    let mut score = Score::default();
    let mut bare = BareScore::default();
    for (lemma, slots) in &attested {
        let keys = keys_for(&by_lemma, lemma);
        let prims = primary.get(lemma);
        for (slot, forms) in slots.iter().enumerate() {
            if forms.is_empty() {
                continue;
            }
            let degree = if slot == 0 {
                Degree::Comparative
            } else {
                Degree::Superlative
            };
            let label = if slot == 0 {
                "Comparative"
            } else {
                "Superlative"
            };
            let produced: BTreeSet<String> =
                keys.iter().map(|v| English::adverb(v, &degree)).collect();
            let bare_said = English::adverb(lemma, &degree);
            let union_hit = score_slot(&mut score, forms, &produced);
            if !union_hit {
                let shown: Vec<&str> = forms.iter().map(|s| s.as_str()).collect();
                writer.write_record(&[
                    lemma.clone(),
                    label.to_string(),
                    shown.join("|"),
                    bare_said.clone(),
                ])?;
            }

            let prim = prims.and_then(|p| p[slot].as_deref());
            let primary_reachable = prim.is_some_and(|p| produced.contains(p));
            if !score_bare(
                &mut bare,
                &bare_said,
                lemma,
                prim,
                union_hit,
                primary_reachable,
            ) {
                let category = if union_hit && primary_reachable {
                    "standard-demoted-to-_n"
                } else {
                    "bare-miss"
                };
                bare_writer.write_record(&[
                    lemma.clone(),
                    label.to_string(),
                    prim.unwrap_or("").to_string(),
                    bare_said,
                    category.to_string(),
                ])?;
            }
        }
    }

    writer.flush()?;
    bare_writer.flush()?;
    println!(
        "adverb form check: {}/{} slots ({}); bare-form primary {}/{} (demoted {}); misses in {}",
        score.matched_slots,
        score.total_slots,
        score.percent_display(),
        bare.bare_primary_hits,
        bare.total,
        bare.demoted,
        output_path.display()
    );
    Ok((score, bare))
}

// The accuracy harness is `checks`-feature-gated and is the number every rule/
// table change must carry, yet `cargo test --workspace` runs with default
// features and never exercises it. These dump-free unit tests pin the pure
// scoring/key-derivation logic so an inverted hit test or a dropped bare lemma
// fails CI (the `test` job runs `--features checks`), not just a local metric.
#[cfg(all(test, feature = "checks"))]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn score_slot_counts_hits_and_variant_gap() {
        let mut score = Score::default();
        // attested {a,b}, produced {a,c}: a hit (a is produced), one unreachable (b).
        assert!(score_slot(&mut score, &set(&["a", "b"]), &set(&["a", "c"])));
        assert_eq!(
            (
                score.matched_slots,
                score.total_slots,
                score.unreachable_forms
            ),
            (1, 1, 1)
        );
        // attested {x}, produced {y}: a miss, one more unreachable.
        assert!(!score_slot(&mut score, &set(&["x"]), &set(&["y"])));
        assert_eq!(
            (
                score.matched_slots,
                score.total_slots,
                score.unreachable_forms
            ),
            (1, 2, 2)
        );
        assert!((score.percent() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn keys_for_groups_by_lemma_and_always_includes_the_bare_lemma_once() {
        let table_keys = vec![
            "cat".to_string(),
            "die".to_string(),
            "die_2".to_string(),
            "die_3".to_string(),
        ];
        let by_lemma = published_keys(&table_keys);
        assert_eq!(by_lemma.get("die").map(|v| v.len()), Some(3));
        // The lemma's bare key comes first and is never duplicated by the suffixed set.
        let q = keys_for(&by_lemma, "die");
        assert_eq!(q.first().map(String::as_str), Some("die"));
        assert_eq!(q.iter().filter(|k| *k == "die").count(), 1);
        assert!(q.contains(&"die_2".to_string()) && q.contains(&"die_3".to_string()));
        // A lemma with no table entry still queries its own bare form (rule fallback).
        assert_eq!(keys_for(&by_lemma, "zzz"), vec!["zzz".to_string()]);
    }

    #[test]
    fn percent_display_never_rounds_a_non_perfect_score_to_100() {
        let s = |matched: u64, total: u64| Score {
            matched_slots: matched,
            total_slots: total,
            unreachable_forms: 0,
        };
        // Perfect scores print 100.00%.
        assert_eq!(s(0, 0).percent_display(), "100.00%");
        assert_eq!(s(225900, 225900).percent_display(), "100.00%");
        // 121548/121550 = 99.9984% must NOT display as 100.00% — it gains a decimal.
        assert_eq!(s(121548, 121550).percent_display(), "99.998%");
        // 25123/25125 = 99.992% shows below 100 at two decimals already.
        assert_eq!(s(25123, 25125).percent_display(), "99.99%");
        // An ordinary imperfect score.
        assert_eq!(s(1, 2).percent_display(), "50.00%");
        // Every imperfect score renders strictly below 100.
        for total in [7u64, 101, 999, 25125, 121550, 1_000_000] {
            let d = s(total - 1, total).percent_display();
            assert_ne!(d, "100.00%", "{total}");
            let n: f64 = d.trim_end_matches('%').parse().unwrap();
            assert!(n < 100.0, "{d} for {total}");
        }
    }

    #[test]
    fn bare_is_primary_matches_only_the_first_listed_form() {
        assert!(bare_is_primary("cats", Some("cats")));
        // album/roof: the bare rule form is NOT the standard first-listed plural.
        assert!(!bare_is_primary("alba", Some("albums")));
        assert!(!bare_is_primary("rooves", Some("roofs")));
        assert!(!bare_is_primary("anything", None));
    }

    #[test]
    fn is_overgeneration_flags_inflected_invariants() {
        // Dump attests the slot invariant (primary == base) but the library inflected it.
        assert!(is_overgeneration("sheeps", "sheep", Some("sheep")));
        // The library left it unchanged -> not over-generation.
        assert!(!is_overgeneration("sheep", "sheep", Some("sheep")));
        // A normally-inflected slot (primary != base) is not over-generation.
        assert!(!is_overgeneration("cats", "cat", Some("cats")));
        assert!(!is_overgeneration("cats", "cat", None));
    }

    #[test]
    fn score_bare_separates_primary_demotion_and_overgeneration() {
        // Clean: the bare lemma returns the primary form.
        let mut b = BareScore::default();
        assert!(score_bare(&mut b, "cats", "cat", Some("cats"), true, true));
        assert_eq!(
            (b.bare_primary_hits, b.total, b.demoted, b.overgenerated),
            (1, 1, 0, 0)
        );

        // Demotion: the slot metric still hits (an `_n` key produces the standard
        // form, so it is reachable) but the bare lemma yields a non-primary form
        // (`album` -> `alba`, standard `albums` only at `album_2`).
        let mut b = BareScore::default();
        assert!(!score_bare(
            &mut b,
            "alba",
            "album",
            Some("albums"),
            true,
            true
        ));
        assert_eq!(
            (b.bare_primary_hits, b.total, b.demoted, b.overgenerated),
            (0, 1, 1, 0)
        );

        // Over-generation: the dump says invariant, the library inflected it; this
        // is counted as over-generation, not a demotion.
        let mut b = BareScore::default();
        assert!(!score_bare(
            &mut b,
            "sheeps",
            "sheep",
            Some("sheep"),
            false,
            false
        ));
        assert_eq!(
            (b.bare_primary_hits, b.total, b.demoted, b.overgenerated),
            (0, 1, 0, 1)
        );

        // Buckets are MUTUALLY EXCLUSIVE: an invariant whose base is also reachable
        // via a key (union_hit + primary_reachable) counts as over-generation ONLY,
        // never double-counted as a demotion.
        let mut b = BareScore::default();
        assert!(!score_bare(
            &mut b,
            "sheeps",
            "sheep",
            Some("sheep"),
            true,
            true
        ));
        assert_eq!(
            (b.bare_primary_hits, b.total, b.demoted, b.overgenerated),
            (0, 1, 0, 1)
        );
    }
}
