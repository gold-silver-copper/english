//! Dump schema (the wiktextract fields we deserialize), form/sense tag policy,
//! and the eligibility filters everything upstream relies on.
//!
//! The tag taxonomy is the heart of the emission policy:
//! - [`EXCLUDE_TAGS`]: dead forms, never usable for anything;
//! - [`SOFT_TAGS`]: real but nonstandard variants — kept in candidate pools so
//!   they can inform decisions, but NEVER emitted as output;
//! - plain (no tag from either set): the only forms the library publishes.
//!
//! [`tag_rank`] encodes this as None/Some(1)/Some(0). `EXCLUDE_TAGS` and
//! `SOFT_TAGS` must stay disjoint (asserted by a test); changing either set
//! changes which forms are emitted, so re-run `cargo xtask refresh-data` (and
//! `accuracy`) after editing them.

use csv::Writer;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub static BAD_CHARS: &[&str] = &[
    ".", "/", "&", " ", "'", "-", "#", "@", "`", "*", "%", "(", "!",
];

/// Tags that *disqualify* a form outright: dead or broken forms we never want to
/// emit under any circumstances.
pub static EXCLUDE_TAGS: &[&str] = &["obsolete", "archaic", "error-unknown-tag"];

/// Editorially-*soft* tags: a form carrying one of these is a real but nonstandard
/// variant (dialectal/rare/jocular/etc.). Soft forms are never emitted — only
/// PLAIN forms (carrying neither `EXCLUDE_TAGS` nor `SOFT_TAGS`) become output, so
/// a nonstandard spelling can never shadow the regular rule (e.g. never emit
/// `busses`/`dangerouser`/`ageing`). The tag also feeds the sense-level softness
/// check ([`entry_is_soft`]) that keeps a slang sense off the bare key.
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

/// Sense-level softness: a sense carrying any of these (the form-level
/// exclude/soft sets, plus register tags that only appear on senses) is a
/// nonstandard sense. An entry whose EVERY sense is soft describes a register
/// variant of the lemma, not its standard use — such an entry's regular-equal
/// forms must not reserve the bare key for the rule engine (`bad`: the slang
/// senses inflect `badder`, but bare `bad` belongs to standard `worse`).
pub fn sense_is_soft(tags: &[String]) -> bool {
    tags.iter().any(|t| {
        EXCLUDE_TAGS.contains(&t.as_str())
            || SOFT_TAGS.contains(&t.as_str())
            || matches!(t.as_str(), "slang" | "vulgar" | "derogatory" | "offensive")
    })
}

/// Whether every sense of this entry is soft (see [`sense_is_soft`]).
/// A senseless entry is NOT soft — absence of tags is standard by default.
pub fn entry_is_soft(entry: &Entry) -> bool {
    !entry.senses.is_empty() && entry.senses.iter().all(|s| sense_is_soft(&s.tags))
}

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
/// it (each a `(form, tag_rank)` pair, IN THE DUMP'S LISTING ORDER). Only *plain*
/// forms (`tag_rank == 0`) are ever emitted — soft variants are deliberately
/// ignored, so a nonstandard spelling can never become canonical output (returns
/// `None` when no plain form exists; the caller falls back to the rule). Among the
/// plain forms:
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
/// 2. the FIRST-LISTED form breaks any remaining tie. Wiktionary head templates
///    list the canonical form first, and wiktextract preserves that order — it is
///    editorial data (`abode` before `abid`, `crowed` before `crew`), unlike the
///    lexicographic tiebreak this replaces, which picked whatever sorted first.
///
/// Determinism: listing order is a property of a single entry, so it is fixed for
/// a given dump and only chooses which attested form is the slot's CANONICAL one.
/// Final key numbers come from a signature sort in [`crate::assign`], not from
/// listing order, so reordering the dump's entries can never change the output.
pub fn choose_form(
    candidates: &[(String, u8)],
    regular: &str,
    prefer_regular: bool,
) -> Option<String> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, (_, rank))| *rank == 0)
        .min_by_key(|(i, (form, _))| ((*form == regular) != prefer_regular, *i))
        .map(|(_, (form, _))| form.clone())
}

/// Returns true if the input contains any non-alphabetic character. The
/// explicit [`BAD_CHARS`] list predates the general `is_alphabetic` check and
/// is kept for the early-exit hot path on huge dumps.
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
/// sense-level `tags` — the sole per-sense signal the deterministic generator
/// uses (to tell a soft/slang sense from a standard one). Identity fields
/// (senseid, wikidata, gloss) are no longer read: keys are numbered by form
/// signature, not by sense identity.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Sense {
    /// Sense-level grammatical/register tags (`uncountable`, `slang`, ...).
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub forms: Option<Vec<Forms>>,
    pub lang_code: String,
    #[serde(default)]
    pub senses: Vec<Sense>,
}

/// The three parts of speech this crate tracks.
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

/// Whether a dump entry participates in extraction at all: English, the right
/// part of speech, and a proper single-word lemma. Rejected entries are
/// invisible to every policy downstream — including bare-key reservation — so
/// widening this filter is a data-model change, not a tweak.
pub fn entry_is_proper(entry: &Entry, pos: &str) -> bool {
    if entry.lang_code != "en" {
        return false;
    }

    if entry.pos != pos || !word_is_proper(&entry.word) {
        return false;
    }
    true
}

/// The lemma/form character policy: ASCII-alphabetic single tokens only (no
/// digits, punctuation, or whitespace — periphrastic adjective forms are the
/// one deliberate exception, admitted explicitly at their call site).
///
/// INVARIANT the public API depends on: because this rejects `_`, no emitted
/// lemma ever contains an underscore, which is what makes the `_<n>` sense
/// suffix in `assign::make_key` unambiguous — `English::noun("F2")` can never be
/// confused with a numbered key. Admitting underscores here would break suffix
/// parsing across the `english` crate.
pub fn word_is_proper(word: &str) -> bool {
    !word.is_empty() && word.bytes().all(|b| b.is_ascii_alphabetic())
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

/// Longest common prefix length (legacy suffix-rule analysis tooling).
pub fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count()
}

/// Given singular & plural, extract their suffix transformation. Only used by
/// the offline rule-mining artifact (`analyze_and_write_suffix_rules`) that
/// informs which suffix classes are worth adding to `english-core` — never in
/// the emission path.
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
    fn exclude_and_soft_are_disjoint() {
        use std::collections::HashSet;
        // tag_rank tests EXCLUDE before SOFT, so a tag in both sets would be
        // silently miscategorised; the two must never overlap.
        let excl: HashSet<&str> = EXCLUDE_TAGS.iter().copied().collect();
        let soft: HashSet<&str> = SOFT_TAGS.iter().copied().collect();
        assert!(
            excl.is_disjoint(&soft),
            "EXCLUDE_TAGS and SOFT_TAGS must be disjoint"
        );
    }

    #[test]
    fn word_is_proper_is_ascii_alphabetic_only() {
        assert!(word_is_proper("word"));
        assert!(word_is_proper("CamelCase"));

        for rejected in [
            "",
            "F16",
            "mp3",
            "word_2",
            "a|b",
            "a b",
            "can't",
            "éclair",
            "quote\"",
            "slash\\",
            "hyphen-word",
            "hash#word",
        ] {
            assert!(!word_is_proper(rejected), "{rejected:?} must be rejected");
        }
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
    fn choose_form_prefers_irregular_then_listing_order() {
        // all plain; "reg" is the rule prediction -> the non-regular wins, and the
        // FIRST-LISTED non-regular breaks the tie (listing order is editorial:
        // canonical first), NOT the lexicographically smallest.
        let c = forms(&[("reg", 0), ("zzz", 0), ("kkk", 0)]);
        assert_eq!(choose_form(&c, "reg", false), Some("zzz".into()));
        // prefer_regular=true (the 3sg slot): the rule-equal form wins when present.
        assert_eq!(choose_form(&c, "reg", true), Some("reg".into()));
        // A soft form ahead of a plain one never wins regardless of position.
        let soft_first = forms(&[("aaa", 1), ("bbb", 0)]);
        assert_eq!(choose_form(&soft_first, "reg", false), Some("bbb".into()));
    }

    #[test]
    fn entry_is_soft_only_when_every_sense_is_soft() {
        let soft = |t: &str| Sense {
            tags: vec![t.to_string()],
        };
        let mk = |senses: Vec<Sense>| Entry {
            word: "w".into(),
            pos: "adj".into(),
            forms: None,
            lang_code: "en".into(),
            senses,
        };
        // Every sense soft -> soft entry.
        assert!(entry_is_soft(&mk(vec![soft("slang"), soft("nonstandard")])));
        // One standard sense makes the whole entry standard.
        assert!(!entry_is_soft(&mk(vec![soft("slang"), Sense::default()])));
        // A senseless entry is standard by default.
        assert!(!entry_is_soft(&mk(vec![])));
    }
}
