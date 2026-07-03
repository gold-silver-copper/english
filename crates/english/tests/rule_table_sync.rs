//! Guard on the layering contract between `english-core`'s regular rules and the
//! generated tables: the tables hold only what the rules CANNOT predict.
//!
//! The extractor drops any candidate whose forms equal the rule prediction
//! (`drop_regular`), so after a regeneration this holds by construction. If a rule
//! in `english-core` changes without regenerating the tables, rows the new rule
//! now predicts become redundant and this fails — forcing the regeneration instead
//! of shipping a core/table drift.

use english::__table_entries;
use english_core::grammar::{Form, Number, Person, Tense};
use english_core::EnglishCore;

fn strip_suffix_digits(key: &str) -> &str {
    key.trim_end_matches(|c: char| c.is_ascii_digit())
}

#[test]
fn tables_hold_only_what_the_rules_cannot_predict() {
    let (nouns, verbs, adjs) = __table_entries();
    let mut redundant = Vec::new();

    for (key, plural) in nouns {
        let lemma = strip_suffix_digits(key);
        if EnglishCore::pluralize_noun(lemma) == plural {
            redundant.push(format!("noun {key} -> {plural} (rule already predicts it)"));
        }
    }

    for (key, (third, past, pres_part, past_part)) in verbs {
        let lemma = strip_suffix_digits(key);
        let p = |tense: &Tense, form: &Form| {
            EnglishCore::verb(lemma, &Person::Third, &Number::Singular, tense, form)
        };
        if p(&Tense::Present, &Form::Finite) == third
            && p(&Tense::Past, &Form::Finite) == past
            && p(&Tense::Present, &Form::Participle) == pres_part
            && p(&Tense::Past, &Form::Participle) == past_part
        {
            redundant.push(format!("verb {key} (rule already predicts all four forms)"));
        }
    }

    for (key, (comp, sup)) in adjs {
        let lemma = strip_suffix_digits(key);
        if EnglishCore::comparative(lemma) == comp && EnglishCore::superlative(lemma) == sup {
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
