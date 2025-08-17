use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
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
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run --release rawwiki.jsonl", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    //let input_path = "../../english.jsonl";

    check_noun_plurals(input_path, "noun_plural_check.csv")?;
    check_verb_conjugations(input_path, "verbs_check.csv")?;
    check_adjective_forms(input_path, "adj_check.csv")?;

    extract_verb_conjugations_new(input_path, "verb_conjugations.csv")?;
    extract_irregular_nouns(input_path, "nouns_with_plurals.csv")?;

    extract_irregular_adjectives(input_path, "adjectives.csv")?;
    generate_adjectives_file("adjectives.csv", "adj_array.rs");
    generate_nouns_file("nouns_with_plurals.csv", "noun_array.rs");
    generate_verbs_file("verb_conjugations.csv", "verb_array.rs");
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
struct AdjParts {
    positive: String,
    comparative: String,
    superlative: String,
}

fn extract_irregular_adjectives(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut forms_map: HashMap<String, HashSet<AdjParts>> = HashMap::new();
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

        let infinitive = entry.word.to_lowercase();
        if !forms_map.contains_key(&infinitive) {
            forms_map.insert(infinitive.clone(), HashSet::new());
        }
        let mut adjik = AdjParts::default();
        adjik.positive = infinitive.clone();

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
                    adjik.comparative = entry_form.clone();
                }

                if tags.contains(&"superlative".into()) {
                    adjik.superlative = entry_form.clone();
                }
            }
        }

        let predicted_comparative = EnglishCore::comparative(&infinitive);
        let predicted_superlative = EnglishCore::superlative(&infinitive);
        if adjik.comparative == "" {
            adjik.comparative = predicted_comparative.clone();
        }
        if adjik.superlative == "" {
            adjik.superlative = predicted_superlative.clone();
        }

        forms_map
            .get_mut(&infinitive)
            .unwrap()
            .insert(adjik.clone());
    }
    for (inf, setik) in forms_map.iter_mut() {
        let predicted_comparative = EnglishCore::comparative(&inf);
        let predicted_superlative = EnglishCore::superlative(&inf);

        let mut predicted_adj = AdjParts::default();
        predicted_adj.positive = inf.clone();
        predicted_adj.comparative = predicted_comparative.clone();
        predicted_adj.superlative = predicted_superlative.clone();
        if setik.is_empty() {
            continue;
        }

        let mut index = match setik.remove(&predicted_adj) {
            true => 2,
            false => 1,
        };
        let mut sorted_vec: Vec<AdjParts> = setik.clone().into_iter().collect();
        sorted_vec.sort(); // uses Ord for sorting
        for thing in sorted_vec.iter() {
            let word_key = if index == 1 {
                inf.clone()
            } else {
                format!("{inf}{index}")
            };
            //positive,comparative,superlative
            let keyd_struct = [
                word_key.clone(),
                thing.comparative.clone(),
                thing.superlative.clone(),
            ];
            index += 1;
            writer.write_record(&keyd_struct)?;
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

pub fn check_noun_plurals(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    use english::*;
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["wiki_single", "wiktionary_plural"])?;

    let mut total_counter = 0;
    let mut match_counter = 0;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Only proper English nouns
        if !entry_is_proper(&entry, "noun") {
            continue;
        }
        let lowercased_entry = entry.word.to_lowercase();

        // Gather all plural forms from Wiktionary
        let mut wiktionary_plurals = Vec::new();
        if let Some(forms) = entry.forms {
            for form in forms {
                if form.tags.contains(&"plural".into())
                //    && !contains_bad_tag(form.tags.clone())
                //   && word_is_proper(&form.form)
                {
                    wiktionary_plurals.push(form.form.to_lowercase());
                }
            }
        }
        if wiktionary_plurals.is_empty() {
            continue;
        }

        // Try base word and numbered variants
        let mut variants = vec![lowercased_entry.clone()];
        for i in 2..=9 {
            variants.push(format!("{}{}", lowercased_entry, i));
        }

        for wiki_plural in &wiktionary_plurals {
            let wiki_plural = wiki_plural.to_lowercase();
            total_counter += 1;
            let mut matched = false;
            for variant in &variants {
                let generated_plural = English::noun(&variant, &Number::Plural);
                matched = generated_plural == wiki_plural;
                if matched {
                    match_counter += 1;
                    break;
                }
            }
            if !matched {
                writer.write_record(&[lowercased_entry.clone(), wiki_plural.clone()])?;
            }
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    println!("total match amount: {} / {}", match_counter, total_counter);
    Ok(())
}

pub fn check_verb_conjugations(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    use english::*;
    let (reader, mut writer) = base_setup(input_path, output_path);

    writer.write_record(&["wiktionary_form", "person", "number", "tense", "form"])?;

    let mut total_counter = 0;
    let mut match_counter = 0;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Only proper English verbs
        if !entry_is_proper(&entry, "verb") {
            continue;
        }

        let lowercased_entry = entry.word.to_lowercase();

        // Collect (form, person, number, tense, form_type) from Wiktionary
        let mut wiktionary_forms = Vec::new();
        if let Some(forms) = entry.forms {
            for form in forms {
                let tags = form
                    .tags
                    .iter()
                    .map(|t| t.to_lowercase())
                    .collect::<Vec<_>>();
                let form_str = form.form.to_lowercase();

                // Skip bad data
                if form_str == "dubious"
                    || contains_bad_tag(form.tags.clone())
                    || !word_is_proper(&form.form)
                {
                    continue;
                }

                // Determine grammatical properties
                //Only check third person, first and second are always the infinitive
                let person = if tags.contains(&"first-person".into()) {
                    continue;
                } else if tags.contains(&"second-person".into()) {
                    continue;
                } else {
                    Person::Third
                };

                //only check singular, plural is always same as singular except for third singular present
                let number = if tags.contains(&"plural".into()) {
                    continue;
                } else {
                    Number::Singular
                };

                let tense = if tags.contains(&"present".into()) {
                    Tense::Present
                } else if tags.contains(&"past".into()) {
                    Tense::Past
                } else {
                    Tense::Present
                };

                let form_type = if tags.contains(&"participle".into()) {
                    Form::Participle
                } else if tags.contains(&"infinitive".into()) {
                    continue;
                } else {
                    Form::Finite
                };

                wiktionary_forms.push((form_str, person, number, tense, form_type));
            }
        }

        if wiktionary_forms.is_empty() {
            continue;
        }

        // Try base and numbered variants
        let mut variants = vec![lowercased_entry.clone()];
        for i in 2..=9 {
            variants.push(format!("{}{}", lowercased_entry, i));
        }

        for (wiki_form, person, number, tense, form_type) in wiktionary_forms {
            total_counter += 1;
            let mut matched = false;

            for variant in &variants {
                let generated_form = English::verb(variant, &person, &number, &tense, &form_type);
                matched = generated_form == wiki_form;
                if matched {
                    match_counter += 1;

                    break;
                }
            }

            if !matched {
                writer.write_record(&[
                    wiki_form.clone(),
                    format!("{:?}", person),
                    format!("{:?}", number),
                    format!("{:?}", tense),
                    format!("{:?}", form_type),
                ])?;
            }
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    println!("total match amount: {} / {}", match_counter, total_counter);
    Ok(())
}

pub fn check_adjective_forms(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    use english::*;
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["wiktionary_form", "degree"])?;

    let mut total_counter = 0;
    let mut match_counter = 0;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Only proper English adjectives
        if !entry_is_proper(&entry, "adj") {
            continue;
        }

        let lowercased_entry = entry.word.to_lowercase();

        // Gather all adjective forms from Wiktionary
        let mut wiki_comparative: Option<String> = None;
        let mut wiki_superlative: Option<String> = None;

        if let Some(forms) = entry.forms {
            for form in forms {
                let form_str = form.form.to_lowercase();
                let tags_lower: Vec<String> = form.tags.iter().map(|t| t.to_lowercase()).collect();

                if tags_lower.contains(&"comparative".into()) {
                    wiki_comparative = Some(form_str);
                } else if tags_lower.contains(&"superlative".into()) {
                    wiki_superlative = Some(form_str);
                }
            }
        }

        // If Wiktionary has no comparative or superlative, skip
        if wiki_comparative.is_none() && wiki_superlative.is_none() {
            continue;
        }

        // Try base and numbered variants
        let mut variants = vec![lowercased_entry.clone()];
        for i in 2..=9 {
            variants.push(format!("{}{}", lowercased_entry, i));
        }

        // Comparative
        if let Some(wiki_comp) = &wiki_comparative {
            let wiki_comp = wiki_comp.to_lowercase();
            total_counter += 1;
            let mut matched = false;
            for variant in &variants {
                let generated_comp = English::adj(variant, &Degree::Comparative);
                if generated_comp == wiki_comp {
                    match_counter += 1;
                    matched = true;
                    break;
                }
            }
            if !matched {
                writer.write_record(&[wiki_comp.clone(), "Comparative".into()])?;
            }
        }

        // Superlative
        if let Some(wiki_sup) = &wiki_superlative {
            let wiki_sup = wiki_sup.to_lowercase();
            total_counter += 1;
            let mut matched = false;
            for variant in &variants {
                let generated_sup = English::adj(variant, &Degree::Superlative);
                if generated_sup == wiki_sup {
                    match_counter += 1;
                    matched = true;
                    break;
                }
            }
            if !matched {
                writer.write_record(&[wiki_sup.clone(), "Superlative".into()])?;
            }
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path);
    println!("total match amount: {} / {}", match_counter, total_counter);
    Ok(())
}
