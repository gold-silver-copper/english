use crate::helpers::{
    Entry, Pos, choose_form, entry_is_proper, suffix_rule, tag_rank, word_is_proper,
};
use crate::registry::{Candidate, Lock};
use csv::{ReaderBuilder, WriterBuilder};
use english_core::*;
use std::collections::{BTreeMap, HashMap, HashSet};

/// A refresh that would retire more than this fraction of a part-of-speech's active
/// lock rows as "absent" is treated as a bad/partial dump: reaping is skipped and
/// reported rather than silently gutting the lockfile. Legitimate cross-dump churn
/// is far below this.
const REAP_MAX_FRACTION: f64 = 0.10;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Per-lemma accumulator while scanning the dump: distinct inflection patterns
/// (keyed by their form signature so duplicates merge) plus whether a
/// regular-prediction-equal pattern was seen (and dropped as a runtime fall-through).
#[derive(Default)]
struct LemmaAcc {
    /// emitted-form signature -> candidate (anchors enriched across the entries that
    /// share it). Keyed by `forms.join("|")`, so two entries merge only when they emit
    /// identical forms.
    by_sig: BTreeMap<String, Candidate>,
    had_regular: bool,
}

impl LemmaAcc {
    /// Record one observed inflection pattern (the plain-preferred emitted columns)
    /// for this lemma, attaching the entry's stable anchors. Deduped by the emitted
    /// signature, so two entries merge only when they emit the same forms.
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
    }

    /// Drop the pattern that exactly equals the regular prediction; the rule engine
    /// produces it at runtime, so a table row would add nothing (and `had_regular`
    /// then reserves the bare key for the rule).
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

/// Pick the form to emit for one inflection slot, falling back to the regular rule
/// prediction when no plain candidate is eligible.
fn select_slot(forms: &[(String, u8)], predicted: &str, prefer_regular: bool) -> String {
    choose_form(forms, predicted, prefer_regular).unwrap_or_else(|| predicted.to_string())
}

pub fn extract_irregular_nouns(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
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

        // Gather every eligible plural-tagged form for THIS entry (= this etymology)
        // and emit a single candidate for it, rather than one homograph key per
        // plural spelling. A sense that lists `cacti`/`cactus`/`cactusses` is one
        // identity, not three; genuine homographs arrive as separate entries and
        // stay separated by their etymology/qid/sid anchor.
        let mut plurals: Vec<(String, u8)> = Vec::new();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" || !word_is_proper(&entry_form) {
                    continue;
                }
                if !form.tags.iter().any(|t| t == "plural") {
                    continue;
                }
                let Some(rank) = tag_rank(&form.tags) else {
                    continue;
                };
                plurals.push((entry_form, rank));
            }
        }
        if plurals.is_empty() {
            continue;
        }

        let predicted = EnglishCore::pluralize_noun(&lemma);
        // If Wiktionary attests the regular plural for this sense, the bare lemma
        // key is reserved for the rule engine (had_regular) and any genuine
        // irregular variant is emitted at a higher suffix — never letting a
        // nonstandard plural (`busses`) hijack the bare key. Only when the regular
        // form is *not* attested does the irregular become the bare key
        // (`child` -> `children`, since `childs` never appears).
        // had_regular is tag-invariant: a regular plural counts whether or not it
        // carries a soft tag, so toggling such a tag never flips the bare-key reservation.
        let has_regular = plurals.iter().any(|(f, _rank)| *f == predicted);
        let irregular = choose_form(&plurals, &predicted, false).filter(|f| *f != predicted);
        let acc = by_lemma.entry(lemma).or_default();
        if has_regular {
            acc.had_regular = true;
        }
        if let Some(primary) = irregular {
            acc.observe(vec![primary], &entry);
        }
    }

    let mut resolved: HashSet<String> = HashSet::new();
    for (lemma, mut acc) in by_lemma {
        let predicted_plural = EnglishCore::pluralize_noun(&lemma);
        acc.drop_regular(&[predicted_plural]);
        let (candidates, had_regular) = acc.into_candidates();
        if candidates.is_empty() {
            continue;
        }
        lock.resolve(&lemma, Pos::Noun, candidates, had_regular, date);
        resolved.insert(lemma);
    }
    lock.reap(Pos::Noun, &resolved, date, REAP_MAX_FRACTION);

    println!("Resolved irregular nouns into the lock.");
    Ok(())
}

pub fn extract_irregular_adjectives(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
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

        let mut comp_forms: Vec<(String, u8)> = Vec::new();
        let mut sup_forms: Vec<(String, u8)> = Vec::new();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" || !word_is_proper(&entry_form) {
                    continue;
                }
                let Some(rank) = tag_rank(tags) else {
                    continue;
                };
                if tags.iter().any(|t| t == "comparative") {
                    comp_forms.push((entry_form.clone(), rank));
                }
                if tags.iter().any(|t| t == "superlative") {
                    sup_forms.push((entry_form, rank));
                }
            }
        }

        let predicted_comp = EnglishCore::comparative(&lemma);
        let predicted_sup = EnglishCore::superlative(&lemma);
        let comparative = select_slot(&comp_forms, &predicted_comp, false);
        let superlative = select_slot(&sup_forms, &predicted_sup, false);

        by_lemma
            .entry(lemma)
            .or_default()
            .observe(vec![comparative, superlative], &entry);
    }

    let mut resolved: HashSet<String> = HashSet::new();
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
        lock.resolve(&lemma, Pos::Adj, candidates, had_regular, date);
        resolved.insert(lemma);
    }
    lock.reap(Pos::Adj, &resolved, date, REAP_MAX_FRACTION);

    println!("Resolved irregular adjectives into the lock.");
    Ok(())
}

pub fn extract_verb_conjugations(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
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

        // Gather every eligible form per slot, then pick each deterministically.
        let mut third_forms: Vec<(String, u8)> = Vec::new();
        let mut past_forms: Vec<(String, u8)> = Vec::new();
        let mut pres_part_forms: Vec<(String, u8)> = Vec::new();
        let mut past_part_forms: Vec<(String, u8)> = Vec::new();

        if let Some(forms) = &entry.forms {
            for form in forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" || !word_is_proper(&entry_form) {
                    continue;
                }
                let Some(rank) = tag_rank(tags) else {
                    continue;
                };

                let has = |t: &str| tags.iter().any(|x| x == t);
                if has("third-person") && has("singular") && has("present") {
                    third_forms.push((entry_form.clone(), rank));
                }
                if has("past") && !has("participle") {
                    past_forms.push((entry_form.clone(), rank));
                }
                if has("participle") && has("present") {
                    pres_part_forms.push((entry_form.clone(), rank));
                }
                if has("participle") && has("past") {
                    past_part_forms.push((entry_form, rank));
                }
            }
        }

        let predicted_third = EnglishCore::verb(
            &lemma,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
        );
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

        // Synthesize the regular 3sg when the dump omits it, exactly as we already do
        // for past/participles. A verb's presence must depend only on it being an
        // English verb, never on whether the dump happened to tag a 3sg form —
        // otherwise a missing tag drops the whole verb and renumbers it next refresh.
        // 3sg is regular for almost every verb, so prefer the predicted form when
        // attested; the other slots prefer a genuine irregular over the rule output.
        let third = select_slot(&third_forms, &predicted_third, true);
        let past = select_slot(&past_forms, &predicted_past, false);
        let present_part = select_slot(&pres_part_forms, &predicted_participle, false);
        // Past participle falls back to the chosen past form when unattested.
        let past_part =
            choose_form(&past_part_forms, &predicted_past, false).unwrap_or_else(|| past.clone());

        by_lemma
            .entry(lemma)
            .or_default()
            .observe(vec![third, past, present_part, past_part], &entry);
    }

    let mut resolved: HashSet<String> = HashSet::new();
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
        lock.resolve(&lemma, Pos::Verb, candidates, had_regular, date);
        resolved.insert(lemma);
    }
    lock.reap(Pos::Verb, &resolved, date, REAP_MAX_FRACTION);

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
