use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

pub fn generate_adjectives_file(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let mut entries: Vec<(String, (String, String))> = reader
        .lines()
        .skip(1) // Skip header row
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            Some((
                parts.next()?.trim().to_string(), // positive
                (
                    parts.next()?.trim().to_string(), // comparative
                    parts.next()?.trim().to_string(), // superlative
                ),
            ))
        })
        .collect();

    // Sort by positive form
    entries.sort_by_key(|(pos, _)| pos.clone());

    let mut output = File::create(outputik)?;

    writeln!(output, "/// (comparative, superlative)")?;
    writeln!(output, "static ADJECTIVE_MAP: &[(&str, (&str, &str))] = &[")?;
    for (positive, (comparative, superlative)) in &entries {
        writeln!(
            output,
            "    (\"{}\", (\"{}\", \"{}\")),",
            positive, comparative, superlative
        )?;
    }
    writeln!(output, "];\n")?;

    writeln!(
        output,
        "pub fn get_adjective_forms(positive: &str) -> Option<(&'static str, &'static str)> {{"
    )?;
    writeln!(
        output,
        "    ADJECTIVE_MAP.binary_search_by_key(&positive, |&(k, _)| k)"
    )?;
    writeln!(output, "        .ok()")?;
    writeln!(output, "        .map(|i| ADJECTIVE_MAP[i].1)")?;
    writeln!(output, "}}")?;

    Ok(())
}

pub fn generate_nouns_phf(inputik: &str, outputik: &str) -> std::io::Result<()> {
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

    // Sort by word for determinism (not required by phf, but helps reproducibility)
    pairs.sort_by_key(|(word, _)| word.clone());

    let mut output = File::create(outputik)?;

    // Start file with imports
    writeln!(output, "use phf::phf_map;\n")?;

    writeln!(
        output,
        "pub static PLURAL_MAP: phf::Map<&'static str, &'static str> = phf_map! {{"
    )?;

    for (word, plural) in &pairs {
        writeln!(output, "    \"{}\" => \"{}\",", word, plural)?;
    }

    writeln!(output, "}};\n")?;

    writeln!(
        output,
        "pub fn get_plural(word: &str) -> Option<&'static str> {{ PLURAL_MAP.get(word).copied() }}"
    )?;

    Ok(())
}
