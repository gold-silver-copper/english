use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

static BAD_TAGS: &[&str] = &[
    "obsolete",
    "error-unknown-tag",
    "dialectal",
    "alternative",
    "nonstandard",
];

fn contains_bad_tag(words: Vec<String>) -> bool {
    for word in words {
        if BAD_TAGS.contains(&&*word) {
            return true;
        }
    }
    false
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
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = "../../english.jsonl";
    // extract_irregular_noun_plurals(input_path, "nouns_with_plurals.csv")?;
    // extract_verb_conjugations(input_path, "verb_conjugations.csv")?;
    extract_irregular_adjectives(input_path, "adjectives.csv")?;
    // generate_nouns_file("nouns_with_plurals.csv", "nounsiki.rs");
    // generate_verbs_file("verb_conjugations.csv", "verbsiki.rs");
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
                    && form.form == EnglishCore::pluralize_noun(&entry.word)
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
                    && form.form != EnglishCore::pluralize_noun(&entry.word)
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
            for form in &forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();

                if tags.contains(&"third-person".into())
                    && tags.contains(&"singular".into())
                    && tags.contains(&"present".into())
                    && !contains_bad_tag(tags.clone())
                {
                    has_third = true;
                    forms_map.insert("third_person_singular", entry_form.clone());
                }

                if tags.contains(&"past".into())
                    && !tags.contains(&"participle".into())
                    && !contains_bad_tag(tags.clone())
                {
                    forms_map.insert("past", entry_form.clone());
                }

                if tags.contains(&"participle".into())
                    && tags.contains(&"present".into())
                    && !contains_bad_tag(tags.clone())
                {
                    forms_map.insert("present_participle", entry_form.clone());
                }

                if tags.contains(&"participle".into())
                    && tags.contains(&"past".into())
                    && !contains_bad_tag(tags.clone())
                {
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
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);
    let mut duplicate_map = HashSet::new();

    let mut writer = Writer::from_path(output_path)?;
    writer.write_record(&["positive", "comparative", "superlative"])?;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.lang_code != "en" {
            continue;
        }

        if entry.pos != "adj" || entry.word.contains(" ") {
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

pub fn generate_nouns_file(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let mut pairs: Vec<(String, String)> = reader
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            Some((
                parts.next()?.trim().to_string(),
                parts.next()?.trim().to_string(),
            ))
        })
        .collect();

    // Sort by the word (key)
    pairs.sort_by_key(|(word, _)| word.clone());

    // Write to a Rust file
    let mut output = File::create(outputik)?;

    writeln!(output, "static PLURAL_MAP: &[(&str, &str)] = &[")?;
    for (word, plural) in &pairs {
        writeln!(output, "    (\"{}\", \"{}\"),", word, plural)?;
    }
    writeln!(output, "];\n")?;

    writeln!(
        output,
        "pub fn get_plural(word: &str) -> Option<&'static str> {{"
    )?;
    writeln!(
        output,
        "    PLURAL_MAP.binary_search_by_key(&word, |&(k, _)| k).ok().map(|i| PLURAL_MAP[i].1)"
    )?;
    writeln!(output, "}}")?;
    Ok(())
}

pub fn generate_verbs_file(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let mut entries: Vec<(String, (String, String, String, String))> = reader
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            Some((
                parts.next()?.trim().to_string(), // infinitive
                (
                    parts.next()?.trim().to_string(), // 3rd person singular
                    parts.next()?.trim().to_string(), // past
                    parts.next()?.trim().to_string(), // present participle
                    parts.next()?.trim().to_string(), // past participle
                ),
            ))
        })
        .collect();

    // Sort by infinitive
    entries.sort_by_key(|(inf, _)| inf.clone());

    let mut output = File::create(outputik)?;

    writeln!(
        output,
        "/// (3rd person singular, past, present participle, past participle)"
    )?;
    writeln!(
        output,
        "static VERB_MAP: &[(&str, (&str, &str, &str, &str))] = &["
    )?;
    for (inf, (third, past, pres_part, past_part)) in &entries {
        writeln!(
            output,
            "    (\"{}\", (\"{}\", \"{}\", \"{}\", \"{}\")),",
            inf, third, past, pres_part, past_part
        )?;
    }
    writeln!(output, "];\n")?;

    writeln!(
        output,
        "pub fn get_verb_forms(infinitive: &str) -> Option<(&'static str, &'static str, &'static str, &'static str)> {{"
    )?;
    writeln!(
        output,
        "    VERB_MAP.binary_search_by_key(&infinitive, |&(k, _)| k)"
    )?;
    writeln!(output, "        .ok()")?;
    writeln!(output, "        .map(|i| VERB_MAP[i].1)")?;
    writeln!(output, "}}")?;

    Ok(())
}
