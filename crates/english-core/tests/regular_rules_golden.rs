//! Golden outputs for the REGULAR rule engine on representative un-tabled words.
//!
//! The `english` crate's generated PHF tables hold ONLY irregular exceptions;
//! every regular word is served by these rules with no table row at all. The
//! `english` crate's `rule_table_sync` test can only catch a rule change that
//! makes an existing bare TABLE row redundant — it is blind to a rule change that
//! breaks the fallback for the far larger set of words that have no row precisely
//! because the old rule predicted them. This golden test pins those fallbacks so
//! such a change fails `cargo test` deterministically, without the Wiktionary
//! dump. (It is NOT an exhaustive correctness spec — after any rule edit, run
//! `cargo xtask refresh-data` then `cargo xtask accuracy` to re-measure.)
//!
//! Every word here is regular AND served by the rule (its rule output is the
//! correct, attested form, so the extractor emits no table row for it).

use english_core::EnglishCore;
use english_core::grammar::{Form, Number, Person, Tense};

fn plural(w: &str) -> String {
    EnglishCore::noun(w, &Number::Plural)
}

fn conjugate(w: &str, tense: Tense, form: Form) -> String {
    EnglishCore::verb(w, &Person::Third, &Number::Singular, &tense, &form)
}

#[test]
fn regular_noun_plurals() {
    for (word, expected) in [
        ("cat", "cats"),
        ("dog", "dogs"),
        ("box", "boxes"),
        ("church", "churches"),
        ("glass", "glasses"),
        ("city", "cities"),
        ("boy", "boys"),
        ("key", "keys"),
        ("bus", "buses"),
    ] {
        assert_eq!(plural(word), expected, "plural of {word}");
    }
}

#[test]
fn regular_verb_conjugations() {
    // (3sg, past, present participle, past participle) — the tuple the verb table
    // stores. `want`/`paint`/`hunt` are the t-ending fallbacks a bad `-ing` rule
    // edit would break (want -> "wantting"); pinned here so it fails loudly.
    for (word, third, past, pres_part, past_part) in [
        ("walk", "walks", "walked", "walking", "walked"),
        ("want", "wants", "wanted", "wanting", "wanted"),
        ("paint", "paints", "painted", "painting", "painted"),
        ("hunt", "hunts", "hunted", "hunting", "hunted"),
        ("watch", "watches", "watched", "watching", "watched"),
        ("try", "tries", "tried", "trying", "tried"),
        ("hope", "hopes", "hoped", "hoping", "hoped"),
        ("stop", "stops", "stopped", "stopping", "stopped"),
        ("carry", "carries", "carried", "carrying", "carried"),
        ("fix", "fixes", "fixed", "fixing", "fixed"),
    ] {
        assert_eq!(
            conjugate(word, Tense::Present, Form::Finite),
            third,
            "{word} 3sg"
        );
        assert_eq!(
            conjugate(word, Tense::Past, Form::Finite),
            past,
            "{word} past"
        );
        assert_eq!(
            conjugate(word, Tense::Present, Form::Participle),
            pres_part,
            "{word} pres.part"
        );
        assert_eq!(
            conjugate(word, Tense::Past, Form::Participle),
            past_part,
            "{word} past.part"
        );
    }
}

#[test]
fn regular_adjective_degrees() {
    for (word, comp, sup) in [
        ("fast", "faster", "fastest"),
        ("happy", "happier", "happiest"),
        ("big", "bigger", "biggest"),
        ("nice", "nicer", "nicest"),
        ("free", "freer", "freest"),
        ("dry", "drier", "driest"),
        ("beautiful", "more beautiful", "most beautiful"),
    ] {
        assert_eq!(
            EnglishCore::comparative(word),
            comp,
            "comparative of {word}"
        );
        assert_eq!(EnglishCore::superlative(word), sup, "superlative of {word}");
    }
}
