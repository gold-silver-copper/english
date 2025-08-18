use csv::ReaderBuilder;
use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fs::File;

use std::io::Write;
use std::io::{BufRead, BufReader};

/// Escape `\` and `"` so we can safely embed strings in Rust source.
fn escape_for_rust(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Parse a string like `"(a, as);(a, ae);(man, men)"` into Vec<(sing, plur)>.
/// Assumes each item is wrapped in parentheses and separated by `;`.
fn parse_pair_list(s: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();

    for raw in s.split(';') {
        let item = raw.trim();
        if item.is_empty() {
            continue;
        }

        // Strip surrounding parentheses if present
        let item = item.strip_prefix('(').unwrap_or(item);
        let item = item.strip_suffix(')').unwrap_or(item);

        // Split once on the first comma into (sing, plur)
        let mut it = item.splitn(2, ',');
        let sing = it.next().map(str::trim).unwrap_or("").to_string();
        let plur = it.next().map(str::trim).unwrap_or("").to_string();

        if sing.is_empty() || plur.is_empty() {
            continue; // skip malformed entries
        }

        // Deduplicate within a row while preserving order
        if seen.insert((sing.clone(), plur.clone())) {
            out.push((sing, plur));
        }
    }
    out
}

/// Reads the grouped CSV (columns: `letter`, `singular_plural_list`)
/// and generates:
/// `pub static INSANE_MAP: &[(&str, &[(&str, &str)])] = &[ ... ];`
pub fn generate_insane_file2(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(inputik)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Keep keys ordered a..z using BTreeMap. We'll also ensure all letters exist.
    let mut grouped: BTreeMap<char, Vec<(String, String)>> =
        ('a'..='z').map(|ch| (ch, Vec::new())).collect();

    for result in rdr.records() {
        let record = result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let letter_field = record.get(0).unwrap_or("").trim();
        let list_field = record.get(1).unwrap_or("").trim();

        // Take the first character of the letter column; skip rows without a valid a..z
        let key = letter_field.chars().next().unwrap_or('\0');
        if !key.is_ascii_lowercase() {
            continue;
        }

        let pairs = parse_pair_list(list_field);
        grouped.entry(key).or_default().extend(pairs);
    }

    // Write Rust file
    let mut out = File::create(outputik)?;
    writeln!(
        out,
        "pub static INSANE_MAP: &[(&str, &[(&str, &str)])] = &["
    )?;

    for (key, pairs) in grouped {
        writeln!(out, "    (\"{}\", &[", key)?;
        for (sing, plur) in pairs {
            let sing = escape_for_rust(&sing);
            let plur = escape_for_rust(&plur);
            writeln!(out, "        (\"{}\", \"{}\"),", sing, plur)?;
        }
        writeln!(out, "    ]),")?;
    }

    writeln!(out, "];")?;
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
