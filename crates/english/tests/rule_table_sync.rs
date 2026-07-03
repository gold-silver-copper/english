//! Guard on the layering contract between `english-core`'s regular rules and the
//! generated tables: a BARE key's row must hold something the rules cannot
//! predict — a bare row equal to the rule's output is dead weight, and its
//! appearance means a core rule changed without regenerating the tables.
//!
//! Two exemptions, both about rows that are load-bearing despite rule-equal
//! content:
//! - Numbered keys: a suffixed row may equal the base lemma's rule prediction
//!   (slang `bad2` -> badder) because the bare key holds a different irregular
//!   (`worse`), so the rule is unreachable — the row is the only way to publish
//!   the forms.
//! - Bare keys WITH numbered siblings: a rule-equal bare row (`ditsy` ->
//!   ditsier, with the periphrastic sense at `ditsy2`) anchors the bare key —
//!   deleting it would let the sibling's forms migrate onto the bare key.
//! A rule-equal bare row with NO siblings has neither excuse: it is dead weight
//! and means a core rule changed without regenerating.

use english::__table_entries;
use english_core::EnglishCore;
use english_core::grammar::{Form, Number, Person, Tense};
use std::collections::HashSet;

fn split_suffix(key: &str) -> Option<&str> {
    let (base, digits) = key.rsplit_once('_')?;
    (!base.is_empty() && !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
        .then_some(base)
}

fn is_bare(key: &str) -> bool {
    split_suffix(key).is_none()
}

fn suffixed_lemmas<'a, T>(entries: &'a [(&'a str, T)]) -> HashSet<&'a str> {
    entries.iter().filter_map(|(k, _)| split_suffix(k)).collect()
}

#[test]
fn sibling_less_bare_rows_hold_only_what_the_rules_cannot_predict() {
    let (nouns, verbs, adjs) = __table_entries();
    let mut redundant = Vec::new();

    let noun_sib = suffixed_lemmas(&nouns);
    for (key, plural) in &nouns {
        if is_bare(key) && !noun_sib.contains(key) && EnglishCore::pluralize_noun(key) == *plural {
            redundant.push(format!("noun {key} -> {plural} (rule already predicts it)"));
        }
    }

    let verb_sib = suffixed_lemmas(&verbs);
    for (key, (third, past, pres_part, past_part)) in &verbs {
        if !is_bare(key) || verb_sib.contains(key) {
            continue;
        }
        let p = |tense: &Tense, form: &Form| {
            EnglishCore::verb(key, &Person::Third, &Number::Singular, tense, form)
        };
        if p(&Tense::Present, &Form::Finite) == *third
            && p(&Tense::Past, &Form::Finite) == *past
            && p(&Tense::Present, &Form::Participle) == *pres_part
            && p(&Tense::Past, &Form::Participle) == *past_part
        {
            redundant.push(format!("verb {key} (rule already predicts all four forms)"));
        }
    }

    let adj_sib = suffixed_lemmas(&adjs);
    for (key, (comp, sup)) in &adjs {
        if is_bare(key)
            && !adj_sib.contains(key)
            && EnglishCore::comparative(key) == *comp
            && EnglishCore::superlative(key) == *sup
        {
            redundant.push(format!("adj {key} -> {comp}/{sup} (rule already predicts it)"));
        }
    }

    assert!(
        redundant.is_empty(),
        "{} table row(s) are redundant with the regular rules — a core rule changed \
         without regenerating: run `cargo xtask refresh-data`:\n  {}",
        redundant.len(),
        redundant.join("\n  ")
    );
}
