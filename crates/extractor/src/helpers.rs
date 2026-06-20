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

/// Tags that *disqualify* a form outright: dead or broken forms we never want to
/// emit under any circumstances.
pub static EXCLUDE_TAGS: &[&str] = &["obsolete", "archaic", "error-unknown-tag"];

/// Editorially-*soft* tags: a form carrying one of these is a real but nonstandard
/// variant (dialectal/rare/jocular/etc.). We keep these forms in the candidate pool
/// — so a later, anchor-based identity pass can expose them as variants — but they
/// are never chosen as the canonical emitted form: a nonstandard spelling must not
/// shadow the regular rule (e.g. never emit `busses`/`dangerouser`/`ageing`). Note
/// `EXCLUDE_TAGS ∪ SOFT_TAGS` equals the legacy `BAD_TAGS` set, so emitted forms are
/// drawn from exactly the same (plain) pool as before — only the selection within it
/// is now deterministic.
pub static SOFT_TAGS: &[&str] = &[
    "dialectal",
    "alternative",
    "nonstandard",
    "humorous",
    "feminine",
    "pronunciation-spelling",
    "rare",
    "dated",
    "informal",
    "sometimes",
    "colloquial",
];

/// Rank a form's tags for selection. Returns `None` if the form is disqualified
/// (an [`EXCLUDE_TAGS`] match), `Some(0)` for a plain form, and `Some(1)` for a
/// form carrying only [`SOFT_TAGS`]. Lower is preferred.
pub fn tag_rank(tags: &[String]) -> Option<u8> {
    if tags.iter().any(|t| EXCLUDE_TAGS.contains(&t.as_str())) {
        return None;
    }
    if tags.iter().any(|t| SOFT_TAGS.contains(&t.as_str())) {
        Some(1)
    } else {
        Some(0)
    }
}

/// Choose the form to emit for one inflection slot from the candidates gathered for
/// it (each a `(form, tag_rank)` pair). Only *plain* forms (`tag_rank == 0`) are ever
/// emitted — soft variants are deliberately ignored, so a nonstandard spelling can
/// never become canonical output (returns `None` when no plain form exists; the
/// caller falls back to the rule). Among the plain forms the choice is fully
/// deterministic and independent of the dump's form-array order:
///
/// 1. `prefer_regular` decides whether a form matching the regular-rule prediction
///    is favored or disfavored:
///    - `false` (past/participle/plural/comparative slots): a form that *differs*
///      from the prediction wins, so a genuine irregular is never shadowed by a
///      rule-equal spelling (`dove` over `dived`, `cacti` over `cactuses`);
///    - `true` (the 3rd-person-singular slot, which is regular `+s/+es` for almost
///      every verb): the predicted form wins when it is attested, so a stray
///      archaic/nonstandard 3sg (`allyeth`, `alibies`) never displaces the regular
///      one — yet a genuinely irregular 3sg (`has`, `does`) is still chosen when the
///      predicted spelling is not attested at all.
/// 2. the lexicographically smallest form breaks any remaining tie.
pub fn choose_form(candidates: &[(String, u8)], regular: &str, prefer_regular: bool) -> Option<String> {
    candidates
        .iter()
        .filter(|(_, rank)| *rank == 0)
        .min_by(|a, b| {
            ((a.0 == regular) != prefer_regular)
                .cmp(&((b.0 == regular) != prefer_regular))
                .then_with(|| a.0.cmp(&b.0))
        })
        .map(|(form, _)| form.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn forms(v: &[(&str, u8)]) -> Vec<(String, u8)> {
        v.iter().map(|(f, r)| (f.to_string(), *r)).collect()
    }

    #[test]
    fn exclude_plus_soft_equals_legacy_bad_tags() {
        use std::collections::HashSet;
        let union: HashSet<&str> = EXCLUDE_TAGS.iter().chain(SOFT_TAGS.iter()).copied().collect();
        let legacy: HashSet<&str> = BAD_TAGS.iter().copied().collect();
        assert_eq!(
            union, legacy,
            "EXCLUDE_TAGS ∪ SOFT_TAGS must equal the legacy BAD_TAGS set (kept in sync by hand)"
        );
        let excl: HashSet<&str> = EXCLUDE_TAGS.iter().copied().collect();
        let soft: HashSet<&str> = SOFT_TAGS.iter().copied().collect();
        assert!(excl.is_disjoint(&soft), "EXCLUDE_TAGS and SOFT_TAGS must be disjoint");
    }

    #[test]
    fn emit_is_plain_only() {
        // "beta" plain (rank 0), "alpha" soft (rank 1): only the plain form is
        // eligible to be emitted — a nonstandard variant never becomes canonical.
        let c = forms(&[("beta", 0), ("alpha", 1)]);
        assert_eq!(choose_form(&c, "reg", false), Some("beta".into()));
        // No plain candidate -> None (caller falls back to the rule).
        let soft = forms(&[("alpha", 1)]);
        assert_eq!(choose_form(&soft, "reg", false), None);
    }

    #[test]
    fn choose_form_prefers_irregular_then_lexicographic() {
        // all plain; "reg" is the rule prediction -> the non-regular wins, then lex.
        let c = forms(&[("reg", 0), ("zzz", 0), ("kkk", 0)]);
        assert_eq!(choose_form(&c, "reg", false), Some("kkk".into()));
        // prefer_regular=true (the 3sg slot): the rule-equal form wins when present.
        assert_eq!(choose_form(&c, "reg", true), Some("reg".into()));
    }
}
