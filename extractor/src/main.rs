use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
mod file_generation;
use file_generation::*;

static BAD_TAGS: &[&str] = &[
    "obsolete",
    "error-unknown-tag",
    "dialectal",
    "alternative",
    "nonstandard",
    "archaic",
    "humorous",
    "feminine",
    "pronunciation-spelling",
    "rare",
    "dated",
    "informal",
    "sometimes",
    "colloquial",
];
static BAD_CHARS: &[&str] = &[".", "/", "&", " ", "'"];

fn contains_bad_tag(words: Vec<String>) -> bool {
    for word in words {
        if BAD_TAGS.contains(&&*word) {
            return true;
        }
    }
    false
}

fn contains_bad_chars(input: &str) -> bool {
    BAD_CHARS.iter().any(|&bad| input.contains(bad))
}
fn contains_number(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

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
    //  etymology_number: Option<u32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = "../../english.jsonl";

    extract_irregular_nouns(input_path, "nouns_with_plurals.csv")?;
    extract_verb_conjugations(input_path, "verb_conjugations.csv")?;
    extract_irregular_adjectives(input_path, "adjectives.csv")?;
    generate_adjectives_file("adjectives.csv", "adjiki.rs");
    generate_nouns_file("nouns_with_plurals.csv", "nounsiki.rs");
    generate_verbs_file("verb_conjugations.csv", "verbsiki.rs");
    Ok(())
}

fn entry_is_proper(entry: &Entry, pos: &str) -> bool {
    if entry.lang_code != "en" {
        return false;
    }

    if entry.pos != pos || !word_is_proper(&entry.word) {
        return false;
    }
    true
}

fn word_is_proper(word: &str) -> bool {
    if contains_bad_chars(&word) || !word.is_ascii() || contains_number(&word) {
        return false;
    }
    true
}

fn base_setup(input_path: &str, output_path: &str) -> (BufReader<File>, Writer<File>) {
    let input = File::open(input_path).unwrap();
    let reader = BufReader::new(input);
    let mut writer = Writer::from_path(output_path).unwrap();
    (reader, writer)
}

/// Extracts irregular noun plurals and writes them to a CSV.
fn extract_irregular_nouns(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut duplicate_map = HashSet::new();
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["word", "plural"])?;

    let mut tag_set = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(e) => {
                println!("{:#?}", e);
                continue;
            }
        };
        if !entry_is_proper(&entry, "noun") {
            continue;
        }

        let mut forms_map = HashMap::new();

        let infinitive = entry.word.to_lowercase();
        let predicted_plural = EnglishCore::pluralize_noun(&infinitive);

        if let Some(forms) = entry.forms {
            for form in &forms {
                let tags = &form.tags;
                tag_set.insert(tags.clone());
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" {
                    continue;
                }
                if entry_form == predicted_plural {
                    duplicate_map.insert(infinitive.clone());
                }

                if tags.contains(&"plural".into())
                    && !contains_bad_tag(tags.clone())
                    && word_is_proper(&entry_form)
                {
                    forms_map.insert("plural", entry_form.clone());
                }
            }
        }

        let gotten = [
            &infinitive,
            forms_map.get("plural").unwrap_or(&predicted_plural),
        ];
        let predicted_struct = [&infinitive, &predicted_plural];

        if predicted_struct == gotten {
            duplicate_map.insert(infinitive.clone());
        }

        if !duplicate_map.contains(&infinitive) {
            duplicate_map.insert(infinitive.clone());
            writer.write_record(&gotten)?;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    Ok(())
}

/// Extracts verb conjugations and writes them to a CSV.
fn extract_verb_conjugations(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut duplicate_map = HashSet::new();
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
            Ok(e) => e,
            Err(e) => {
                println!("{:#?}", e);
                continue;
            }
        };
        if !entry_is_proper(&entry, "verb") {
            continue;
        }

        let mut forms_map = HashMap::new();
        let mut has_third = false;
        let infinitive = entry.word.to_lowercase();

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
                {
                    has_third = true;
                    forms_map.insert("third_person_singular", entry_form.clone());
                }

                if tags.contains(&"past".into()) && !tags.contains(&"participle".into()) {
                    forms_map.insert("past", entry_form.clone());
                }

                if tags.contains(&"participle".into()) && tags.contains(&"present".into()) {
                    forms_map.insert("present_participle", entry_form.clone());
                }

                if tags.contains(&"participle".into()) && tags.contains(&"past".into()) {
                    forms_map.insert("past_participle", entry_form.clone());
                }
            }
        }
        let predicted_third = EnglishCore::verb(
            &infinitive,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
        );
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

/// Extracts irregular noun plurals and writes them to a CSV.
fn extract_irregular_adjectives(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut duplicate_map = HashSet::new();
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["positive", "comparative", "superlative"])?;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(e) => {
                println!("{:#?}", e);
                continue;
            }
        };
        if !entry_is_proper(&entry, "adj") {
            continue;
        }

        let mut forms_map = HashMap::new();

        let infinitive = entry.word.to_lowercase();

        if let Some(forms) = entry.forms {
            for form in &forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" {
                    continue;
                }

                if tags.contains(&"comparative".into()) && !contains_bad_tag(tags.clone()) {
                    forms_map.insert("comparative", entry_form.clone());
                }

                if tags.contains(&"superlative".into()) && !contains_bad_tag(tags.clone()) {
                    forms_map.insert("superlative", entry_form.clone());
                }
            }
        }
        let predicted_comparative = EnglishCore::comparative(&infinitive);
        let predicted_superlative = EnglishCore::superlative(&infinitive);

        let gotten = [
            &infinitive,
            forms_map
                .get("comparative")
                .unwrap_or(&predicted_comparative),
            forms_map
                .get("superlative")
                .unwrap_or(&predicted_superlative),
        ];
        let predicted_struct = [&infinitive, &predicted_comparative, &predicted_superlative];

        if predicted_struct == gotten {
            duplicate_map.insert(infinitive.clone());
        }

        if !duplicate_map.contains(&infinitive) {
            duplicate_map.insert(infinitive.clone());
            writer.write_record(&gotten)?;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    Ok(())
}
