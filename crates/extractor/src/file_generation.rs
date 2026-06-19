use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

// ---------------------------------------------------------------------------
// Pair-based cores. These hold the exact byte formatting of the generated
// tables; both the CSV entry points and the lock-driven generation funnel
// through them so output is identical regardless of source.
// ---------------------------------------------------------------------------

pub fn write_nouns_phf(
    mut pairs: Vec<(String, String)>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
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

pub fn write_verbs_phf(
    mut entries: Vec<(String, (String, String, String, String))>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
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

pub fn write_adjectives_phf(
    mut entries: Vec<(String, (String, String))>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
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

/// Emit the self-describing "which numbered senses exist for this lemma" tables.
/// Each map only contains base lemmas that actually have a disambiguated sibling
/// (a `lemma2`/`lemma3`/... key), mapping the base to the full sorted key list.
pub fn write_variants_phf(
    noun_keys: Vec<String>,
    verb_keys: Vec<String>,
    adj_keys: Vec<String>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut output = File::create(outputik)?;
    writeln!(output, "use phf::phf_map;")?;
    writeln!(output)?;
    writeln!(
        output,
        "// Base lemma -> every explicit (numbered or bare) key the table holds for it,"
    )?;
    writeln!(
        output,
        "// listed only for lemmas with more than one sense. Lets callers discover"
    )?;
    writeln!(
        output,
        "// disambiguation suffixes instead of hard-coding magic integers."
    )?;
    write_one_variants_map(&mut output, "NOUN_VARIANTS", "noun_variants", noun_keys)?;
    write_one_variants_map(&mut output, "VERB_VARIANTS", "verb_variants", verb_keys)?;
    write_one_variants_map(&mut output, "ADJ_VARIANTS", "adj_variants", adj_keys)?;
    Ok(())
}

fn write_one_variants_map(
    output: &mut File,
    static_name: &str,
    fn_name: &str,
    keys: Vec<String>,
) -> std::io::Result<()> {
    use std::collections::BTreeMap;
    let mut by_base: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for k in keys {
        let base = strip_trailing_digits(&k).to_string();
        by_base.entry(base).or_default().push(k);
    }
    // Keep only multi-sense bases; sort their key lists.
    let multi: BTreeMap<String, Vec<String>> = by_base
        .into_iter()
        .filter_map(|(base, mut ks)| {
            // a base is "multi-sense" if it has a numbered key (suffix >= 2)
            let has_numbered = ks.iter().any(|k| k != &base);
            if has_numbered {
                ks.sort();
                ks.dedup();
                Some((base, ks))
            } else {
                None
            }
        })
        .collect();

    writeln!(output)?;
    writeln!(
        output,
        "pub static {static_name}: phf::Map<&'static str, &'static [&'static str]> = phf_map! {{"
    )?;
    for (base, ks) in &multi {
        let list = ks
            .iter()
            .map(|k| format!("\"{k}\""))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(output, "    \"{base}\" => &[{list}],")?;
    }
    writeln!(output, "}};")?;
    writeln!(output)?;
    writeln!(
        output,
        "pub fn {fn_name}(base: &str) -> Option<&'static [&'static str]> {{ {static_name}.get(base).copied() }}"
    )?;
    Ok(())
}

fn strip_trailing_digits(word: &str) -> &str {
    word.trim_end_matches(|c: char| c.is_ascii_digit())
}

/// Emit the optional dictionary tables: sense key -> every Wiktionary definition
/// for that homograph. Each map mirrors the `*_meanings` accessor naming.
pub fn write_dictionary_phf(
    noun: std::collections::BTreeMap<String, Vec<String>>,
    verb: std::collections::BTreeMap<String, Vec<String>>,
    adj: std::collections::BTreeMap<String, Vec<String>>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut output = File::create(outputik)?;
    writeln!(output, "use phf::phf_map;")?;
    writeln!(output)?;
    writeln!(
        output,
        "// Sense key (e.g. \"lie2\") -> Wiktionary definitions for that homograph."
    )?;
    writeln!(
        output,
        "// Covers the keys present in the inflection tables; empty for fully-regular words."
    )?;
    write_one_meanings_map(&mut output, "NOUN_MEANINGS", "noun_meanings", noun)?;
    write_one_meanings_map(&mut output, "VERB_MEANINGS", "verb_meanings", verb)?;
    write_one_meanings_map(&mut output, "ADJ_MEANINGS", "adj_meanings", adj)?;
    Ok(())
}

fn write_one_meanings_map(
    output: &mut File,
    static_name: &str,
    fn_name: &str,
    dict: std::collections::BTreeMap<String, Vec<String>>,
) -> std::io::Result<()> {
    writeln!(output)?;
    writeln!(
        output,
        "pub static {static_name}: phf::Map<&'static str, &'static [&'static str]> = phf_map! {{"
    )?;
    for (key, defs) in &dict {
        let defs = dedup_preserve_order(defs);
        if defs.is_empty() {
            continue;
        }
        let list = defs
            .iter()
            .map(|d| format!("\"{}\"", escape_rust_string(d)))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(output, "    \"{key}\" => &[{list}],")?;
    }
    writeln!(output, "}};")?;
    writeln!(output)?;
    writeln!(
        output,
        "pub fn {fn_name}(key: &str) -> Option<&'static [&'static str]> {{ {static_name}.get(key).copied() }}"
    )?;
    Ok(())
}

fn dedup_preserve_order(defs: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    defs.iter()
        .filter(|d| !d.is_empty())
        .filter(|d| seen.insert((*d).clone()))
        .cloned()
        .collect()
}

/// Escape a gloss into valid Rust string-literal content (UTF-8 preserved).
fn escape_rust_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' | '\r' | '\t' => out.push(' '),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// CSV entry points (legacy path; the extractor still writes intermediate CSVs).
// ---------------------------------------------------------------------------

pub fn generate_nouns_phf(
    inputik: impl AsRef<Path>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let pairs: Vec<(String, String)> = reader
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

    write_nouns_phf(pairs, outputik)
}

pub fn generate_verbs_phf(
    inputik: impl AsRef<Path>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let entries: Vec<(String, (String, String, String, String))> = reader
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

    write_verbs_phf(entries, outputik)
}

pub fn generate_adjectives_phf(
    inputik: impl AsRef<Path>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let entries: Vec<(String, (String, String))> = reader
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

    write_adjectives_phf(entries, outputik)
}
