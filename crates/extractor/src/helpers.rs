use csv::Writer;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub static BAD_TAGS: &[&str] = &[
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
pub static BAD_CHARS: &[&str] = &[
    ".", "/", "&", " ", "'", "-", "#", "@", "`", "*", "%", "(", "!",
];

pub fn contains_bad_tag(words: Vec<String>) -> bool {
    for word in words {
        if BAD_TAGS.contains(&&*word) {
            return true;
        }
    }
    false
}

/// Returns true if the input contains any non-alphabetic character.
pub fn contains_bad_chars(input: &str) -> bool {
    for x in BAD_CHARS.iter() {
        if input.contains(x) {
            return true;
        }
    }
    !input.chars().all(|c| c.is_alphabetic())
}

pub fn contains_number(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

#[derive(Debug, Deserialize)]
pub struct Forms {
    pub form: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A single Wiktionary sense as emitted by wiktextract. We only deserialize the
/// fields that can anchor a *stable* identity for a homograph across dumps:
/// author-authored `senseid`s (durable by Wiktionary convention), Wikidata QIDs,
/// and the gloss text (kept only as a human review aid).
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Sense {
    #[serde(default)]
    pub senseid: Vec<String>,
    #[serde(default)]
    pub wikidata: Vec<String>,
    #[serde(default)]
    pub glosses: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub forms: Option<Vec<Forms>>,
    pub lang_code: String,
    /// Ordinal of the numbered Etymology section this entry sits under. Distinct
    /// numbered etymologies arrive as *separate* top-level entries, so this is the
    /// single best per-entry signal of "which homograph" this is. wiktextract
    /// serialized it as an int historically and as a string after ~2026, so we
    /// accept either (or absent).
    #[serde(default, deserialize_with = "de_etym_number")]
    pub etymology_number: Option<u32>,
    #[serde(default)]
    pub senses: Vec<Sense>,
}

/// Accept `etymology_number` as a JSON integer, a JSON string, or absent/null.
fn de_etym_number<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;
    Ok(match value {
        Value::Number(n) => n.as_u64().and_then(|x| u32::try_from(x).ok()),
        Value::String(s) => s.trim().parse::<u32>().ok(),
        _ => None,
    })
}

/// The three parts of speech this crate tracks. Used as a stable key component in
/// the assignment lockfiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Pos {
    Adj,
    Noun,
    Verb,
}

impl Pos {
    pub fn as_str(self) -> &'static str {
        match self {
            Pos::Adj => "adj",
            Pos::Noun => "noun",
            Pos::Verb => "verb",
        }
    }

    pub fn parse(s: &str) -> Option<Pos> {
        match s {
            "adj" => Some(Pos::Adj),
            "noun" => Some(Pos::Noun),
            "verb" => Some(Pos::Verb),
            _ => None,
        }
    }
}

impl Entry {
    /// Deterministic, stablest QID an editor attached to any of this entry's senses.
    pub fn lowest_qid(&self) -> Option<String> {
        self.senses
            .iter()
            .flat_map(|s| s.wikidata.iter())
            .filter(|q| !q.is_empty())
            .min()
            .cloned()
    }

    /// Deterministic, stablest author-authored senseid (e.g. `en:to_rest`).
    pub fn lowest_senseid(&self) -> Option<String> {
        self.senses
            .iter()
            .flat_map(|s| s.senseid.iter())
            .filter(|s| !s.is_empty())
            .min()
            .cloned()
    }

    /// First non-empty gloss, kept purely as a review aid in the lockfile.
    pub fn first_gloss(&self) -> Option<String> {
        self.senses
            .iter()
            .flat_map(|s| s.glosses.iter())
            .find(|g| !g.is_empty())
            .cloned()
    }

    /// Every non-empty gloss across this entry's senses, in order — the raw
    /// material for the optional dictionary tables.
    pub fn all_glosses(&self) -> Vec<String> {
        self.senses
            .iter()
            .flat_map(|s| s.glosses.iter())
            .filter(|g| !g.is_empty())
            .cloned()
            .collect()
    }
}

#[derive(Debug, Default, Eq, Hash, PartialEq, Clone, Ord, PartialOrd)]
pub struct AdjParts {
    pub positive: String,
    pub comparative: String,
    pub superlative: String,
}

#[derive(Debug, Default, Eq, Hash, PartialEq, Clone, Ord, PartialOrd)]
pub struct VerbParts {
    pub inf: String,
    pub third: String,
    pub past: String,
    pub present_part: String,
    pub past_part: String,
}

pub fn entry_is_proper(entry: &Entry, pos: &str) -> bool {
    if entry.lang_code != "en" {
        return false;
    }

    if entry.pos != pos || !word_is_proper(&entry.word) {
        return false;
    }
    true
}

pub fn word_is_proper(word: &str) -> bool {
    if contains_bad_chars(&word) || !word.is_ascii() || contains_number(&word) {
        return false;
    }
    true
}

pub fn base_setup(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> (BufReader<File>, Writer<File>) {
    let input = File::open(input_path).unwrap();
    let reader = BufReader::new(input);
    let writer = Writer::from_path(output_path).unwrap();
    (reader, writer)
}

/// Find the longest common prefix length
pub fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count()
}

/// Given singular & plural, extract their suffix transformation
pub fn suffix_rule(singular: &str, plural: &str) -> (String, String) {
    let prefix_len = common_prefix_len(singular, plural);
    let (singular_suffix, plural_suffix) = if prefix_len > 0 {
        (&singular[prefix_len - 1..], &plural[prefix_len - 1..])
    } else {
        (&singular[prefix_len..], &plural[prefix_len..])
    };

    (singular_suffix.to_string(), plural_suffix.to_string())
}
