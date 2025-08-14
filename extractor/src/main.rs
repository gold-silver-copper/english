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
static BAD_CHARS: &[&str] = &[".", "/", "&", " ", "'", "-", "#", "@", "`", "*"];

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
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = "../../english.jsonl";

    extract_verb_conjugations_new(input_path, "verb_conjugations.csv")?;
    extract_irregular_nouns(input_path, "nouns_with_plurals.csv")?;

    extract_irregular_adjectives(input_path, "adjectives.csv")?;
    generate_adjectives_file("adjectives.csv", "adjiki.rs");
    generate_nouns_file("nouns_with_plurals.csv", "nounsiki.rs");
    generate_verbs_file("verb_conjugations.csv", "verbsiki.rs");

    /*   */
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
    let mut forms_map: HashMap<String, HashSet<String>> = HashMap::new();

    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["word", "plural"])?;

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

        let infinitive = entry.word.to_lowercase();

        if !forms_map.contains_key(&infinitive) {
            forms_map.insert(infinitive.clone(), HashSet::new());
        }

        let mut plural_found = false;
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
                        .unwrap()
                        .insert(entry_form.clone());
                }
            }
        }
    }

    for (inf, setik) in forms_map.iter_mut() {
        let predicted_plural = EnglishCore::pluralize_noun(&inf);
        if setik.is_empty() {
            continue;
        }
        let alr_cont = setik.remove(&predicted_plural);
        let mut index = match alr_cont {
            true => 2,
            false => 1,
        };
        let mut sorted_vec: Vec<String> = setik.clone().into_iter().collect();
        sorted_vec.sort(); // uses Ord for sorting
        for thing in sorted_vec.iter() {
            let word_key = if index == 1 {
                inf.clone()
            } else {
                format!("{inf}{index}")
            };
            let keyd_struct = [word_key.clone(), thing.clone()];

            if index < 10 {
                writer.write_record(&keyd_struct)?;
            }
            index += 1;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    Ok(())
}
#[derive(Debug, Default, Eq, Hash, PartialEq, Clone, Ord, PartialOrd)]
struct VerbParts {
    inf: String,
    third: String,
    past: String,
    present_part: String,
    past_part: String,
}

/// Extracts verb conjugations and writes them to a CSV.
fn extract_verb_conjugations_new(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
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
            Ok(e) => e,
            Err(e) => {
                println!("{:#?}", e);
                continue;
            }
        };
        if !entry_is_proper(&entry, "verb") {
            continue;
        }

        let mut has_third = false;
        let infinitive = entry.word.to_lowercase();
        if !forms_map.contains_key(&infinitive) {
            forms_map.insert(infinitive.clone(), HashSet::new());
        }
        let mut verbik = VerbParts::default();
        verbik.inf = infinitive.clone();
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
                    verbik.third = entry_form.clone();
                }

                if tags.contains(&"past".into()) && !tags.contains(&"participle".into()) {
                    verbik.past = entry_form.clone();
                }

                if tags.contains(&"participle".into()) && tags.contains(&"present".into()) {
                    verbik.present_part = entry_form.clone();
                }

                if tags.contains(&"participle".into()) && tags.contains(&"past".into()) {
                    verbik.past_part = entry_form.clone();
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

        if verbik.past == "" {
            verbik.past = predicted_past.clone();
        }
        if verbik.past_part == "" {
            verbik.past_part = predicted_past.clone();
        }
        if verbik.present_part == "" {
            verbik.present_part = predicted_participle.clone();
        }

        if has_third {
            forms_map
                .get_mut(&infinitive)
                .unwrap()
                .insert(verbik.clone());
        }
    }
    for (inf, setik) in forms_map.iter_mut() {
        let predicted_third = EnglishCore::verb(
            &inf,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
        );
        let predicted_past = EnglishCore::verb(
            &inf,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite,
        );
        let predicted_participle = EnglishCore::verb(
            &inf,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle,
        );

        let mut predicted_verb = VerbParts::default();
        predicted_verb.inf = inf.clone();
        predicted_verb.third = predicted_third.clone();
        predicted_verb.past = predicted_past.clone();
        predicted_verb.past_part = predicted_past.clone();
        predicted_verb.present_part = predicted_participle.clone();
        if setik.is_empty() {
            continue;
        }

        let mut index = match setik.remove(&predicted_verb) {
            true => 2,
            false => 1,
        };
        let mut sorted_vec: Vec<VerbParts> = setik.clone().into_iter().collect();
        sorted_vec.sort(); // uses Ord for sorting
        for thing in sorted_vec.iter() {
            let word_key = if index == 1 {
                inf.clone()
            } else {
                format!("{inf}{index}")
            };
            //infinitive,third_person_singular,past,present_participle,past_participle
            let keyd_struct = [
                word_key.clone(),
                thing.third.clone(),
                thing.past.clone(),
                thing.present_part.clone(),
                thing.past_part.clone(),
            ];
            index += 1;
            writer.write_record(&keyd_struct)?;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    Ok(())
}

/// Extracts irregular noun plurals and writes them to a CSV.
fn extract_irregular_adjectives(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut duplicate_key_set = HashSet::new();
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
                if !word_is_proper(&entry_form) || contains_bad_tag(tags.clone()) {
                    continue;
                }

                if tags.contains(&"comparative".into()) {
                    forms_map.insert("comparative", entry_form.clone());
                }

                if tags.contains(&"superlative".into()) {
                    forms_map.insert("superlative", entry_form.clone());
                }
            }
        }
        let predicted_comparative = EnglishCore::comparative(&infinitive);
        let predicted_superlative = EnglishCore::superlative(&infinitive);

        match forms_map.get("comparative") {
            Some(_) => (),
            None => {
                duplicate_key_set.insert(infinitive.clone());
                continue;
            }
        }
        match forms_map.get("superlative") {
            Some(_) => (),
            None => {
                duplicate_key_set.insert(infinitive.clone());
                continue;
            }
        }

        let gotten = [
            &infinitive,
            forms_map.get("comparative").unwrap(),
            forms_map.get("superlative").unwrap(),
        ];
        let predicted_struct = [&infinitive, &predicted_comparative, &predicted_superlative];

        if predicted_struct == gotten {
            duplicate_key_set.insert(infinitive.clone());
        }

        if !duplicate_key_set.contains(&infinitive) {
            duplicate_key_set.insert(infinitive.clone());
            writer.write_record(&gotten)?;
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    Ok(())
}
