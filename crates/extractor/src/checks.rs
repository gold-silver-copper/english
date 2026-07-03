//! Dump-driven accuracy measurement (feature `checks`, which links the `english`
//! crate so the CURRENT committed tables + rule engine are what gets measured).
//!
//! For every plain-tagged form Wiktionary attests, ask the library to produce
//! that slot and count whether any sense-numbered variant of the lemma
//! reproduces it. Soft-tagged (nonstandard/dialectal/...) and excluded
//! (obsolete/...) forms are NOT counted against the library: by policy it never
//! emits them. The per-POS percentages are the headline metric — run
//! `cargo xtask accuracy` before and after any rule/table change so the change
//! carries a number, not an anecdote. Misses are written to per-POS CSVs for
//! eyeballing.

#[cfg(feature = "checks")]
use crate::helpers::{Entry, base_setup, entry_is_proper, tag_rank, word_is_proper};
use std::error::Error;
#[cfg(feature = "checks")]
use std::io::BufRead;
use std::path::Path;

/// One slot's score: how many plain attested forms the library reproduced.
#[derive(Debug, Clone, Copy, Default)]
pub struct Score {
    pub matched: u64,
    pub total: u64,
}

#[cfg_attr(not(feature = "checks"), allow(dead_code))]
impl Score {
    fn hit(&mut self) {
        self.matched += 1;
        self.total += 1;
    }
    fn miss(&mut self) {
        self.total += 1;
    }
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            100.0 * self.matched as f64 / self.total as f64
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
        let verb = check_verb_conjugations(filtered_json_path, artifacts_dir.join("verbs_check.csv"))?;
        let adj = check_adjective_forms(filtered_json_path, artifacts_dir.join("adj_check.csv"))?;

        println!("\naccuracy (plain attested forms reproduced by the library):");
        for (name, s) in [("noun plural", noun), ("verb forms", verb), ("adj comp/sup", adj)] {
            println!(
                "  {name:>12}: {:>7}/{:<7} = {:.2}%",
                s.matched,
                s.total,
                s.percent()
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

/// The library reaches a homograph sense through its numbered key; a form counts
/// as reproduced if ANY published variant of the lemma produces it.
#[cfg(feature = "checks")]
fn numbered_variants(word: &str) -> Vec<String> {
    let mut variants = vec![word.to_string()];
    for i in 2..=9 {
        variants.push(format!("{word}{i}"));
    }
    variants
}

#[cfg(feature = "checks")]
pub fn check_noun_plurals(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<Score, Box<dyn Error>> {
    use english::*;

    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path.as_ref(), output_path);
    writer.write_record(&["wiki_singular", "wiki_plural", "library_said"])?;

    let mut score = Score::default();

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

        let mut attested = Vec::new();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let f = form.form.to_lowercase();
                if f == "dubious" || !word_is_proper(&f) {
                    continue;
                }
                if form.tags.iter().any(|t| t == "plural") && tag_rank(&form.tags) == Some(0) {
                    attested.push(f);
                }
            }
        }

        for wiki_plural in &attested {
            let variants = numbered_variants(&lemma);
            let produced: Vec<String> = variants
                .iter()
                .map(|v| English::noun(v, &Number::Plural))
                .collect();
            if produced.iter().any(|p| p == wiki_plural) {
                score.hit();
            } else {
                score.miss();
                writer.write_record(&[lemma.clone(), wiki_plural.clone(), produced[0].clone()])?;
            }
        }
    }

    writer.flush()?;
    println!(
        "noun plural check: {}/{} ({:.2}%); misses in {}",
        score.matched,
        score.total,
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
    writer.write_record(&["lemma", "wiki_form", "slot", "library_said"])?;

    let mut score = Score::default();

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

        // (attested form, slot label, person, number, tense, form)
        let mut targets: Vec<(String, &'static str, Person, Number, Tense, Form)> = Vec::new();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let f = form.form.to_lowercase();
                if f == "dubious" || !word_is_proper(&f) || tag_rank(&form.tags) != Some(0) {
                    continue;
                }
                let has = |t: &str| form.tags.iter().any(|x| x == t);
                if has("third-person") && has("singular") && has("present") {
                    targets.push((f.clone(), "3sg", Person::Third, Number::Singular, Tense::Present, Form::Finite));
                } else if has("participle") && has("present") {
                    targets.push((f.clone(), "pres_part", Person::Third, Number::Singular, Tense::Present, Form::Participle));
                } else if has("participle") && has("past") {
                    targets.push((f.clone(), "past_part", Person::Third, Number::Singular, Tense::Past, Form::Participle));
                } else if has("past") {
                    targets.push((f.clone(), "past", Person::Third, Number::Singular, Tense::Past, Form::Finite));
                }
            }
        }

        for (wiki_form, slot, person, number, tense, form_type) in targets {
            let variants = numbered_variants(&lemma);
            let produced: Vec<String> = variants
                .iter()
                .map(|v| English::verb(v, &person, &number, &tense, &form_type))
                .collect();
            if produced.iter().any(|p| *p == wiki_form) {
                score.hit();
            } else {
                score.miss();
                writer.write_record(&[lemma.clone(), wiki_form, slot.to_string(), produced[0].clone()])?;
            }
        }
    }

    writer.flush()?;
    println!(
        "verb form check: {}/{} ({:.2}%); misses in {}",
        score.matched,
        score.total,
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
    writer.write_record(&["lemma", "wiki_form", "degree", "library_said"])?;

    let mut score = Score::default();

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

        let mut comps = Vec::new();
        let mut sups = Vec::new();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let f = form.form.to_lowercase();
                if f == "dubious" || tag_rank(&form.tags) != Some(0) {
                    continue;
                }
                if form.tags.iter().any(|t| t == "comparative") {
                    comps.push(f.clone());
                }
                if form.tags.iter().any(|t| t == "superlative") {
                    sups.push(f);
                }
            }
        }

        let variants = numbered_variants(&lemma);
        for (attested, degree, label) in [
            (&comps, Degree::Comparative, "Comparative"),
            (&sups, Degree::Superlative, "Superlative"),
        ] {
            for wiki_form in attested {
                let produced: Vec<String> = variants
                    .iter()
                    .map(|v| English::adj(v, &degree))
                    .collect();
                if produced.iter().any(|p| p == wiki_form) {
                    score.hit();
                } else {
                    score.miss();
                    writer.write_record(&[
                        lemma.clone(),
                        wiki_form.clone(),
                        label.to_string(),
                        produced[0].clone(),
                    ])?;
                }
            }
        }
    }

    writer.flush()?;
    println!(
        "adj form check: {}/{} ({:.2}%); misses in {}",
        score.matched,
        score.total,
        score.percent(),
        output_path.display()
    );
    Ok(score)
}
