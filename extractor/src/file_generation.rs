use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

pub fn generate_verbs_phf(inputik: &str, outputik: &str) -> std::io::Result<()> {
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

    // Sort for determinism
    entries.sort_by_key(|(inf, _)| inf.clone());

    let mut output = File::create(outputik)?;

    writeln!(output, "use phf::phf_map;")?;
    writeln!(output)?;
    writeln!(
        output,
        "/// (3rd person singular, past, present participle, past participle)"
    )?;
    writeln!(
        output,
        "pub static VERB_MAP: phf::Map<&'static str, (&'static str, &'static str, &'static str, &'static str)> = phf_map! {{"
    )?;

    for (inf, (third, past, pres_part, past_part)) in &entries {
        writeln!(
            output,
            "    \"{}\" => (\"{}\", \"{}\", \"{}\", \"{}\"),",
            inf, third, past, pres_part, past_part
        )?;
    }

    writeln!(output, "}};")?;
    writeln!(output)?;
    writeln!(
        output,
        "pub fn get_verb_forms(infinitive: &str) -> Option<(&'static str, &'static str, &'static str, &'static str)> {{"
    )?;
    writeln!(output, "    VERB_MAP.get(infinitive).copied()")?;
    writeln!(output, "}}")?;

    Ok(())
}

pub fn generate_adjectives_phf(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let mut entries: Vec<(String, (String, String))> = reader
        .lines()
        .skip(1) // Skip header
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

    // Sort for determinism
    entries.sort_by_key(|(pos, _)| pos.clone());

    let mut output = File::create(outputik)?;

    writeln!(output, "use phf::phf_map;")?;
    writeln!(output)?;
    writeln!(output, "/// (comparative, superlative)")?;
    writeln!(
        output,
        "pub static ADJECTIVE_MAP: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {{"
    )?;

    for (positive, (comparative, superlative)) in &entries {
        writeln!(
            output,
            "    \"{}\" => (\"{}\", \"{}\"),",
            positive, comparative, superlative
        )?;
    }

    writeln!(output, "}};")?;
    writeln!(output)?;
    writeln!(
        output,
        "pub fn get_adjective_forms(positive: &str) -> Option<(&'static str, &'static str)> {{"
    )?;
    writeln!(output, "    ADJECTIVE_MAP.get(positive).copied()")?;
    writeln!(output, "}}")?;

    Ok(())
}
