use crate::helpers::{
    Entry, Pos, contains_bad_tag, entry_is_proper, suffix_rule, word_is_proper,
};
use crate::registry::{Assignment, Candidate, Lock};

/// Accumulated `key -> Wiktionary definitions` for one part of speech.
pub type Definitions = BTreeMap<String, Vec<String>>;

/// Attribute each resolved key's definitions (looked up by form signature, since
/// that is what survives `resolve`) into the per-POS dictionary accumulator.
fn record_definitions(
    dict: &mut Definitions,
    gloss_by_sig: &HashMap<String, Vec<String>>,
    assignments: &[Assignment],
) {
    for a in assignments {
        if let Some(g) = gloss_by_sig.get(&a.forms.join("|"))
            && !g.is_empty()
        {
            dict.entry(a.key.clone()).or_default().extend(g.iter().cloned());
        }
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use english_core::*;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Per-lemma accumulator while scanning the dump: distinct inflection patterns
/// (keyed by their form signature so duplicates merge) plus whether a
/// regular-prediction-equal pattern was seen (and dropped as a runtime fall-through).
#[derive(Default)]
struct LemmaAcc {
    /// signature -> candidate (anchors enriched across the entries that share it)
    by_sig: BTreeMap<String, Candidate>,
    had_regular: bool,
}

impl LemmaAcc {
    /// Record one observed inflection pattern for this lemma, attaching the
    /// entry's stable anchors.
    fn observe(&mut self, forms: Vec<String>, entry: &Entry) {
        let sig = forms.join("|");
        let c = self
            .by_sig
            .entry(sig)
            .or_insert_with(|| Candidate::new(forms));
        if c.qid.is_none() {
            c.qid = entry.lowest_qid();
        }
        if c.sid.is_none() {
            c.sid = entry.lowest_senseid();
        }
        if c.etym.is_none() {
            c.etym = entry.etymology_number;
        }
        if c.gloss.is_none() {
            c.gloss = entry.first_gloss();
        }
        c.glosses.extend(entry.all_glosses());
    }

    /// Drop the pattern that exactly equals the regular prediction; it is produced
    /// at runtime by the rule engine, so we never emit a table row for it.
    fn drop_regular(&mut self, predicted: &[String]) {
        let sig = predicted.join("|");
        if self.by_sig.remove(&sig).is_some() {
            self.had_regular = true;
        }
    }

    fn into_candidates(self) -> (Vec<Candidate>, bool) {
        (self.by_sig.into_values().collect(), self.had_regular)
    }
}

fn open_reader(path: &Path) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

pub fn extract_irregular_nouns(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
    dict: &mut Definitions,
    date: &str,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let mut by_lemma: BTreeMap<String, LemmaAcc> = BTreeMap::new();

    let reader = open_reader(input_path);
    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        if !entry_is_proper(&entry, "noun") {
            continue;
        }

        let lemma = entry.word.to_lowercase();
        let acc = by_lemma.entry(lemma).or_default();

        if let Some(forms) = &entry.forms {
            for form in forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" {
                    continue;
                }
                if !word_is_proper(&entry_form) || contains_bad_tag(tags.clone()) {
                    continue;
                }

                if tags.contains(&"plural".into()) {
                    acc.observe(vec![entry_form], &entry);
                }
            }
        }
    }

    for (lemma, mut acc) in by_lemma {
        let predicted_plural = EnglishCore::pluralize_noun(&lemma);
        acc.drop_regular(&[predicted_plural]);
        let (candidates, had_regular) = acc.into_candidates();
        if candidates.is_empty() {
            continue;
        }
        let gloss_by_sig: HashMap<String, Vec<String>> = candidates
            .iter()
            .map(|c| (c.forms.join("|"), c.glosses.clone()))
            .collect();
        let assignments = lock.resolve(&lemma, Pos::Noun, candidates, had_regular, date);
        record_definitions(dict, &gloss_by_sig, &assignments);
    }

    println!("Resolved irregular nouns into the lock.");
    Ok(())
}

pub fn extract_irregular_adjectives(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
    dict: &mut Definitions,
    date: &str,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let mut by_lemma: BTreeMap<String, LemmaAcc> = BTreeMap::new();
    let reader = open_reader(input_path);

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry_is_proper(&entry, "adj") {
            continue;
        }

        let lemma = entry.word.to_lowercase();
        let mut comparative = String::new();
        let mut superlative = String::new();

        if let Some(forms) = &entry.forms {
            for form in forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" {
                    continue;
                }
                if !word_is_proper(&entry_form) || contains_bad_tag(tags.clone()) {
                    continue;
                }

                if tags.contains(&"comparative".into()) && comparative.is_empty() {
                    comparative = entry_form.clone();
                }

                if tags.contains(&"superlative".into()) && superlative.is_empty() {
                    superlative = entry_form.clone();
                }
            }
        }

        if comparative.is_empty() {
            comparative = EnglishCore::comparative(&lemma);
        }
        if superlative.is_empty() {
            superlative = EnglishCore::superlative(&lemma);
        }

        by_lemma
            .entry(lemma)
            .or_default()
            .observe(vec![comparative, superlative], &entry);
    }

    for (lemma, mut acc) in by_lemma {
        let predicted = [
            EnglishCore::comparative(&lemma),
            EnglishCore::superlative(&lemma),
        ];
        acc.drop_regular(&predicted);
        let (candidates, had_regular) = acc.into_candidates();
        if candidates.is_empty() {
            continue;
        }
        let gloss_by_sig: HashMap<String, Vec<String>> = candidates
            .iter()
            .map(|c| (c.forms.join("|"), c.glosses.clone()))
            .collect();
        let assignments = lock.resolve(&lemma, Pos::Adj, candidates, had_regular, date);
        record_definitions(dict, &gloss_by_sig, &assignments);
    }

    println!("Resolved irregular adjectives into the lock.");
    Ok(())
}

pub fn extract_verb_conjugations(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
    dict: &mut Definitions,
    date: &str,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let mut by_lemma: BTreeMap<String, LemmaAcc> = BTreeMap::new();
    let reader = open_reader(input_path);

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry_is_proper(&entry, "verb") {
            continue;
        }

        let lemma = entry.word.to_lowercase();
        // "to be" has too many forms for the (3sg, past, pres-part, past-part)
        // shape and is handled directly by english-core.
        if lemma == "be" {
            continue;
        }

        let mut has_third_person = false;
        let mut third = String::new();
        let mut past = String::new();
        let mut present_part = String::new();
        let mut past_part = String::new();

        if let Some(forms) = &entry.forms {
            for form in forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if !word_is_proper(&entry_form) || contains_bad_tag(tags.clone()) {
                    continue;
                }

                if tags.contains(&"third-person".into())
                    && tags.contains(&"singular".into())
                    && tags.contains(&"present".into())
                    && !has_third_person
                {
                    has_third_person = true;
                    third = entry_form.clone();
                }

                if tags.contains(&"past".into())
                    && !tags.contains(&"participle".into())
                    && past.is_empty()
                {
                    past = entry_form.clone();
                }

                if tags.contains(&"participle".into())
                    && tags.contains(&"present".into())
                    && present_part.is_empty()
                {
                    present_part = entry_form.clone();
                }

                if tags.contains(&"participle".into())
                    && tags.contains(&"past".into())
                    && past_part.is_empty()
                {
                    past_part = entry_form.clone();
                }
            }
        }

        let predicted_past = EnglishCore::verb(
            &lemma,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite,
        );
        let predicted_participle = EnglishCore::verb(
            &lemma,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle,
        );

        if past.is_empty() {
            past = predicted_past;
        }
        if past_part.is_empty() {
            past_part = past.clone();
        }
        if present_part.is_empty() {
            present_part = predicted_participle;
        }

        if has_third_person {
            by_lemma
                .entry(lemma)
                .or_default()
                .observe(vec![third, past, present_part, past_part], &entry);
        }
    }

    for (lemma, mut acc) in by_lemma {
        let predicted = [
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Present,
                &Form::Finite,
            ),
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Past,
                &Form::Finite,
            ),
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Present,
                &Form::Participle,
            ),
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Past,
                &Form::Finite,
            ),
        ];
        acc.drop_regular(&predicted);
        let (candidates, had_regular) = acc.into_candidates();
        if candidates.is_empty() {
            continue;
        }
        let gloss_by_sig: HashMap<String, Vec<String>> = candidates
            .iter()
            .map(|c| (c.forms.join("|"), c.glosses.clone()))
            .collect();
        let assignments = lock.resolve(&lemma, Pos::Verb, candidates, had_regular, date);
        record_definitions(dict, &gloss_by_sig, &assignments);
    }

    println!("Resolved verb conjugations into the lock.");
    Ok(())
}

pub fn filter_english_entries(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);
    let mut output = File::create(output_path)?;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        if entry.lang_code != "en" {
            continue;
        }

        writeln!(output, "{}", line)?;
    }

    println!("Filtered dataset saved to {}", output_path.display());
    Ok(())
}

pub fn strip_trailing_number(word: &str) -> &str {
    word.trim_end_matches(|c: char| c.is_ascii_digit())
}

pub fn analyze_and_write_suffix_rules(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().from_reader(BufReader::new(file));

    let mut frequency: HashMap<(String, String), usize> = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        let singular_raw = record.get(0).unwrap();
        let plural = record.get(1).unwrap();

        let singular = strip_trailing_number(singular_raw);
        let pair = suffix_rule(singular, plural);
        *frequency.entry(pair).or_insert(0) += 1;
    }

    let mut frequency_rows: Vec<_> = frequency.into_iter().collect();
    frequency_rows.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0.0.cmp(&b.0.0))
            .then_with(|| a.0.1.cmp(&b.0.1))
    });

    let mut writer = WriterBuilder::new().from_path(output_path)?;
    writer.write_record(&["singular_suffix", "plural_suffix", "count"])?;

    for ((singular_suffix, plural_suffix), count) in frequency_rows {
        writer.write_record(&[singular_suffix, plural_suffix, count.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}
