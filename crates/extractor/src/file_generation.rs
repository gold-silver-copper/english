//! PHF table emission. BYTE STABILITY IS LOAD-BEARING here: the
//! `tables_round_trip_committed_output` test parses the committed tables and
//! re-emits them, requiring a byte-identical result — any formatting change
//! (whitespace, ordering, headers) must be committed together with regenerated
//! tables (`cargo xtask refresh-data`). Entries are sorted by key before writing
//! so output is independent of input order.

use std::fs::File;
use std::io::Write;
use std::path::Path;

// ---------------------------------------------------------------------------
// Table writers. These hold the exact byte formatting of the generated tables;
// the `generate_tables` path funnels the extracted candidates through them.
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

pub fn write_adverbs_phf(
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
        "pub static ADVERB_MAP: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {{"
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
        "pub fn get_adverb_forms(positive: &str) -> Option<(&'static str, &'static str)> {{"
    )?;
    writeln!(output, "    ADVERB_MAP.get(positive).copied()")?;
    writeln!(output, "}}")?;

    Ok(())
}
