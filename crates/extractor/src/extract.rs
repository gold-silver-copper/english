use crate::helpers::{
    AdjParts, Entry, VerbParts, base_setup, contains_bad_tag, entry_is_proper, suffix_rule,
    word_is_proper,
};
use csv::{ReaderBuilder, WriterBuilder};
use english_core::*;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn extract_irregular_nouns(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let mut forms_map: HashMap<String, HashSet<String>> = HashMap::new();

    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["word", "plural"])?;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(error) => {
                println!("{:#?}", error);
                continue;
            }
        };

        if !entry_is_proper(&entry, "noun") {
            continue;
        }

        let infinitive = entry.word.to_lowercase();
        forms_map.entry(infinitive.clone()).or_default();

        if let Some(forms) = entry.forms {
            for form in &forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" {
                    continue;
                }
                if !word_is_proper(&entry_form) || contains_bad_tag(tags.clone()) {
                    continue;
                }

                if tags.contains(&"plural".into()) {
                    forms_map
                        .get_mut(&infinitive)
                        .expect("noun entry should exist")
                        .insert(entry_form.clone());
                }
            }
        }
    }

    for (infinitive, forms) in &mut forms_map {
        let predicted_plural = EnglishCore::pluralize_noun(infinitive);
        if forms.is_empty() {
            continue;
        }

        let mut index = if forms.remove(&predicted_plural) {
            2
        } else {
            1
        };
        let mut sorted_forms: Vec<String> = forms.clone().into_iter().collect();
        sorted_forms.sort();

        for form in &sorted_forms {
            let word_key = if index == 1 {
                infinitive.clone()
            } else {
                format!("{infinitive}{index}")
            };
            if index < 10 {
                writer.write_record(&[word_key, form.clone()])?;
            }
            index += 1;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path.display());
    Ok(())
}

pub fn extract_irregular_adjectives(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let mut forms_map: HashMap<String, HashSet<AdjParts>> = HashMap::new();
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["positive", "comparative", "superlative"])?;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(error) => {
                println!("{:#?}", error);
                continue;
            }
        };
        if !entry_is_proper(&entry, "adj") {
            continue;
        }

        let infinitive = entry.word.to_lowercase();
        forms_map.entry(infinitive.clone()).or_default();
        let mut adjective = AdjParts {
            positive: infinitive.clone(),
            ..AdjParts::default()
        };

        if let Some(forms) = entry.forms {
            for form in &forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" {
                    continue;
                }
                if !word_is_proper(&entry_form) || contains_bad_tag(tags.clone()) {
                    continue;
                }

                if tags.contains(&"comparative".into()) && adjective.comparative.is_empty() {
                    adjective.comparative = entry_form.clone();
                }

                if tags.contains(&"superlative".into()) && adjective.superlative.is_empty() {
                    adjective.superlative = entry_form.clone();
                }
            }
        }

        let predicted_comparative = EnglishCore::comparative(&infinitive);
        let predicted_superlative = EnglishCore::superlative(&infinitive);
        if adjective.comparative.is_empty() {
            adjective.comparative = predicted_comparative.clone();
        }
        if adjective.superlative.is_empty() {
            adjective.superlative = predicted_superlative.clone();
        }

        forms_map
            .get_mut(&infinitive)
            .expect("adjective entry should exist")
            .insert(adjective);
    }

    for (infinitive, forms) in &mut forms_map {
        let predicted = AdjParts {
            positive: infinitive.clone(),
            comparative: EnglishCore::comparative(infinitive),
            superlative: EnglishCore::superlative(infinitive),
        };
        if forms.is_empty() {
            continue;
        }

        let mut index = if forms.remove(&predicted) { 2 } else { 1 };
        let mut sorted_forms: Vec<AdjParts> = forms.clone().into_iter().collect();
        sorted_forms.sort();

        for form in &sorted_forms {
            let word_key = if index == 1 {
                infinitive.clone()
            } else {
                format!("{infinitive}{index}")
            };
            writer.write_record(&[word_key, form.comparative.clone(), form.superlative.clone()])?;
            index += 1;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path.display());
    Ok(())
}

pub fn extract_verb_conjugations(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let mut forms_map: HashMap<String, HashSet<VerbParts>> = HashMap::new();
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&[
        "infinitive",
        "third_person_singular",
        "past",
        "present_participle",
        "past_participle",
    ])?;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(error) => {
                println!("{:#?}", error);
                continue;
            }
        };
        if !entry_is_proper(&entry, "verb") {
            continue;
        }

        let infinitive = entry.word.to_lowercase();
        forms_map.entry(infinitive.clone()).or_default();

        let mut has_third_person = false;
        let mut verb = VerbParts {
            inf: infinitive.clone(),
            ..VerbParts::default()
        };

        if verb.inf == "be" {
            continue;
        }

        if let Some(forms) = entry.forms {
            for form in &forms {
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
                    verb.third = entry_form.clone();
                }

                if tags.contains(&"past".into())
                    && !tags.contains(&"participle".into())
                    && verb.past.is_empty()
                {
                    verb.past = entry_form.clone();
                }

                if tags.contains(&"participle".into())
                    && tags.contains(&"present".into())
                    && verb.present_part.is_empty()
                {
                    verb.present_part = entry_form.clone();
                }

                if tags.contains(&"participle".into())
                    && tags.contains(&"past".into())
                    && verb.past_part.is_empty()
                {
                    verb.past_part = entry_form.clone();
                }
            }
        }

        let predicted_past = EnglishCore::verb(
            &infinitive,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite,
        );
        let predicted_participle = EnglishCore::verb(
            &infinitive,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle,
        );

        if verb.past.is_empty() {
            verb.past = predicted_past.clone();
        }
        if verb.past_part.is_empty() {
            verb.past_part = verb.past.clone();
        }
        if verb.present_part.is_empty() {
            verb.present_part = predicted_participle.clone();
        }

        if has_third_person {
            forms_map
                .get_mut(&infinitive)
                .expect("verb entry should exist")
                .insert(verb);
        }
    }

    for (infinitive, forms) in &mut forms_map {
        let predicted = VerbParts {
            inf: infinitive.clone(),
            third: EnglishCore::verb(
                infinitive,
                &Person::Third,
                &Number::Singular,
                &Tense::Present,
                &Form::Finite,
            ),
            past: EnglishCore::verb(
                infinitive,
                &Person::Third,
                &Number::Singular,
                &Tense::Past,
                &Form::Finite,
            ),
            present_part: EnglishCore::verb(
                infinitive,
                &Person::Third,
                &Number::Singular,
                &Tense::Present,
                &Form::Participle,
            ),
            past_part: EnglishCore::verb(
                infinitive,
                &Person::Third,
                &Number::Singular,
                &Tense::Past,
                &Form::Finite,
            ),
        };
        if forms.is_empty() {
            continue;
        }

        let mut index = if forms.remove(&predicted) { 2 } else { 1 };
        let mut sorted_forms: Vec<VerbParts> = forms.clone().into_iter().collect();
        sorted_forms.sort();

        for form in &sorted_forms {
            let word_key = if index == 1 {
                infinitive.clone()
            } else {
                format!("{infinitive}{index}")
            };
            writer.write_record(&[
                word_key,
                form.third.clone(),
                form.past.clone(),
                form.present_part.clone(),
                form.past_part.clone(),
            ])?;
            index += 1;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path.display());
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
