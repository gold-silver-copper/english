use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct Forms {
    form: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    word: String,
    pos: String,
    forms: Option<Vec<Forms>>,
    lang_code: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = "../../english.jsonl";
    extract_irregular_noun_plurals(input_path, "nouns_with_plurals.csv")?;
    extract_verb_conjugations(input_path, "verb_conjugations.csv")?;
    Ok(())
}

/// Extracts irregular noun plurals and writes them to a CSV.
fn extract_irregular_noun_plurals(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);

    let mut writer = Writer::from_path(output_path)?;
    writer.write_record(&["word", "plural"])?;

    let mut normal_plural_nouns = HashSet::new();
    let mut seen_pairs = HashSet::new();

    // First pass: identify nouns with regular plurals
    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.pos != "noun" || entry.word.contains(" ") {
            continue;
        }
        if entry.lang_code != "en" {
            continue;
        }

        if let Some(forms) = entry.forms {
            for form in forms {
                if form.tags.contains(&"plural".to_string())
                    && form.form == English::pluralize_noun(&entry.word)
                {
                    normal_plural_nouns.insert(entry.word.clone());
                }
            }
        }
    }

    // Second pass: collect irregular plurals
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);
    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.lang_code != "en" {
            continue;
        }

        if entry.pos != "noun"
            || entry.word.contains(" ")
            || normal_plural_nouns.contains(&entry.word)
        {
            continue;
        }

        if let Some(forms) = entry.forms {
            for form in forms {
                if form.tags.contains(&"plural".to_string())
                    && form.form != English::pluralize_noun(&entry.word)
                    && seen_pairs.insert((entry.word.clone(), form.form.clone()))
                {
                    writer.write_record(&[entry.word.clone(), form.form.clone()])?;
                    break;
                }
            }
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    Ok(())
}

/// Extracts verb conjugations and writes them to a CSV.
fn extract_verb_conjugations(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);
    let mut duplicate_map = HashSet::new();

    let mut writer = Writer::from_path(output_path)?;
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
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.lang_code != "en" {
            continue;
        }

        if entry.pos != "verb" || entry.word.contains(" ") {
            continue;
        }

        let mut forms_map = HashMap::new();
        let mut has_third = false;
        let infinitive = entry.word.to_lowercase();

        if let Some(forms) = entry.forms {
            for form in forms {
                let tags = &form.tags;

                if tags.contains(&"third-person".into())
                    && tags.contains(&"singular".into())
                    && tags.contains(&"present".into())
                {
                    has_third = true;
                    forms_map.insert("third_person_singular", form.form.clone());
                }

                if tags.contains(&"past".into()) && !tags.contains(&"participle".into()) {
                    forms_map.insert("past", form.form.clone());
                }

                if tags.contains(&"participle".into()) && tags.contains(&"present".into()) {
                    forms_map.insert("present_participle", form.form.clone());
                }

                if tags.contains(&"participle".into()) && tags.contains(&"past".into()) {
                    forms_map.insert("past_participle", form.form.clone());
                }
            }
        }
        let predicted_third = English::verb(
            &infinitive,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
        );
        let predicted_past = English::verb(
            &infinitive,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite,
        );
        let predicted_participle = English::verb(
            &infinitive,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle,
        );
        let gotten = [
            &infinitive,
            forms_map
                .get("third_person_singular")
                .unwrap_or(&predicted_third),
            forms_map.get("past").unwrap_or(&predicted_past),
            forms_map
                .get("present_participle")
                .unwrap_or(&predicted_participle),
            forms_map.get("past_participle").unwrap_or(&predicted_past),
        ];
        let predicted_struct = [
            &infinitive,
            &predicted_third,
            &predicted_past,
            &predicted_participle,
            &predicted_past,
        ];

        if predicted_struct == gotten {
            duplicate_map.insert(infinitive.clone());
        }

        if has_third && !duplicate_map.contains(&infinitive) {
            duplicate_map.insert(infinitive.clone());
            writer.write_record(&gotten)?;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    Ok(())
}
