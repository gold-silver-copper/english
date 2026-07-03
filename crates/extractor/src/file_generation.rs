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

// ---------------------------------------------------------------------------
// Variants tables: alternate plain attested forms per emitted key, keyed by
// slot. Recognition payload only — never identity, never stability-guaranteed.
// Inputs arrive as (key, sorted (slot, form) pairs) from Lock::emittable_variants.
// ---------------------------------------------------------------------------

fn slot_forms(variants: &[(u8, String)], slot: u8) -> Vec<&str> {
    variants
        .iter()
        .filter(|(s, _)| *s == slot)
        .map(|(_, f)| f.as_str())
        .collect()
}

fn slice_literal(forms: &[&str]) -> String {
    if forms.is_empty() {
        "&[]".to_string()
    } else {
        format!(
            "&[{}]",
            forms.iter().map(|f| format!("\"{f}\"")).collect::<Vec<_>>().join(", ")
        )
    }
}

pub fn write_noun_variants_phf(
    mut entries: Vec<(String, Vec<(u8, String)>)>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut output = File::create(outputik)?;
    writeln!(output, "use phf::phf_map;")?;
    writeln!(output)?;
    writeln!(
        output,
        "/// Alternate plain attested plurals per emitted key (canonical form excluded)."
    )?;
    writeln!(
        output,
        "pub static PLURAL_VARIANTS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {{"
    )?;
    for (key, variants) in &entries {
        writeln!(output, "    \"{}\" => {},", key, slice_literal(&slot_forms(variants, 0)))?;
    }
    writeln!(output, "}};")?;
    Ok(())
}

pub fn write_verb_variants_phf(
    mut entries: Vec<(String, Vec<(u8, String)>)>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut output = File::create(outputik)?;
    writeln!(output, "use phf::phf_map;")?;
    writeln!(output)?;
    writeln!(
        output,
        "/// Alternate plain attested forms per emitted key: (3sg, past, present participle, past participle)."
    )?;
    writeln!(
        output,
        "pub static VERB_VARIANTS: phf::Map<&'static str, (&'static [&'static str], &'static [&'static str], &'static [&'static str], &'static [&'static str])> = phf_map! {{"
    )?;
    for (key, variants) in &entries {
        writeln!(
            output,
            "    \"{}\" => ({}, {}, {}, {}),",
            key,
            slice_literal(&slot_forms(variants, 0)),
            slice_literal(&slot_forms(variants, 1)),
            slice_literal(&slot_forms(variants, 2)),
            slice_literal(&slot_forms(variants, 3)),
        )?;
    }
    writeln!(output, "}};")?;
    Ok(())
}

pub fn write_adj_variants_phf(
    mut entries: Vec<(String, Vec<(u8, String)>)>,
    outputik: impl AsRef<Path>,
) -> std::io::Result<()> {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut output = File::create(outputik)?;
    writeln!(output, "use phf::phf_map;")?;
    writeln!(output)?;
    writeln!(
        output,
        "/// Alternate plain attested forms per emitted key: (comparative, superlative)."
    )?;
    writeln!(
        output,
        "pub static ADJECTIVE_VARIANTS: phf::Map<&'static str, (&'static [&'static str], &'static [&'static str])> = phf_map! {{"
    )?;
    for (key, variants) in &entries {
        writeln!(
            output,
            "    \"{}\" => ({}, {}),",
            key,
            slice_literal(&slot_forms(variants, 0)),
            slice_literal(&slot_forms(variants, 1)),
        )?;
    }
    writeln!(output, "}};")?;
    Ok(())
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
