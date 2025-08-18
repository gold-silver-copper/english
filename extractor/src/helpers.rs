use csv::Writer;
use english_core::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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
pub static BAD_CHARS: &[&str] = &[".", "/", "&", " ", "'", "-", "#", "@", "`", "*"];

pub fn contains_bad_tag(words: Vec<String>) -> bool {
    for word in words {
        if BAD_TAGS.contains(&&*word) {
            return true;
        }
    }
    false
}

pub fn contains_bad_chars(input: &str) -> bool {
    BAD_CHARS.iter().any(|&bad| input.contains(bad))
}
pub fn contains_number(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

#[derive(Debug, Deserialize)]
pub struct Forms {
    pub form: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub forms: Option<Vec<Forms>>,
    pub lang_code: String,
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

pub fn base_setup(input_path: &str, output_path: &str) -> (BufReader<File>, Writer<File>) {
    let input = File::open(input_path).unwrap();
    let reader = BufReader::new(input);
    let mut writer = Writer::from_path(output_path).unwrap();
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
