//! Guards against "obvious" regular-rule fixes that silently regress common
//! irregulars.
//!
//! `english-core`'s regular rules deliberately over-generalize (`play` -> the
//! rule would give `plaies`; `help` -> `helpping`; `chief` -> `chieves`). Those
//! look like bugs and are tempting to "fix" by tightening the rules — adding an
//! `-ay`/`-oy`/`-ey` guard to the verb lists, a CVC gate to `p`/`g` doubling,
//! dropping `f -> ves`. Every attested word is nonetheless correct at runtime
//! today, because the extractor tables any word whose attested form differs from
//! the (wrong) rule prediction.
//!
//! The catch is the bare-key RESERVATION policy (`extractor::LemmaAcc::drop_regular`):
//! when a lemma attests both an irregular form and a regularized variant, and a
//! rule PREDICTS that regularized variant, the variant reserves the bare key and
//! the irregular is demoted to a `_<n>` key. The over-general rules avoid this by
//! predicting a form that matches NOTHING (`paied`, `saied`), so no reservation
//! fires and the deterministic sort keeps the standard irregular on the bare key.
//! "Fixing" a rule to predict the real regularized variant (`payed`, `sayed`,
//! `selfs`) makes that variant reserve the bare key — regressing the common word.
//!
//! There is no data signal to fix this safely: on the 2026-07 dump `abide` (want
//! `abided`, the standard regular) and `pay` (want `paid`, the standard irregular)
//! have the IDENTICAL tag profile — both attest a plain regular and a plain
//! irregular past — so any reservation policy that serves `abide -> abided` also
//! serves `pay -> payed`. See the investigation notes on the PR.
//!
//! These tests pin the currently-correct output of the vulnerable words so that a
//! future rule "fix" fails loudly here instead of silently shipping `pay -> payed`
//! / `self -> selfs`. If you are here because one of these failed after editing an
//! `english-core` rule: your edit regressed a common irregular via the reservation
//! policy — that is the trap this file exists to catch.

use english::{Degree, English, Form, Number, Person, Tense};

fn past(word: &str) -> String {
    English::verb(
        word,
        &Person::Third,
        &Number::Singular,
        &Tense::Past,
        &Form::Finite,
    )
}

fn plural(word: &str) -> String {
    English::noun(word, &Number::Plural)
}

#[test]
fn vowel_y_verbs_keep_their_standard_irregular_past() {
    // An `-ay`/`-oy`/`-ey` guard on the verb rule lists (so `play` -> `plays`
    // instead of `plaies`) makes the rule predict `payed`/`sayed`, which are
    // plainly attested and would reserve the bare key. Standard `paid`/`said`
    // must stay on the bare key.
    for (verb, expected) in [
        ("pay", "paid"),
        ("say", "said"),
        ("lay", "laid"),
        ("repay", "repaid"),
        ("overlay", "overlaid"),
    ] {
        assert_eq!(past(verb), expected, "past of {verb}");
    }
}

#[test]
fn cvc_p_verbs_keep_their_standard_irregular_past() {
    // Gating `p`-doubling on a real CVC check (so `help` -> `helped` instead of
    // `helpped`) makes the rule predict `weeped`/`keeped`/`sleeped`; where those
    // are attested they would reserve the bare key. Standard `wept`/`kept`/…
    // must stay.
    for (verb, expected) in [
        ("weep", "wept"),
        ("keep", "kept"),
        ("sleep", "slept"),
        ("sweep", "swept"),
    ] {
        assert_eq!(past(verb), expected, "past of {verb}");
    }
}

#[test]
fn f_ves_nouns_keep_their_standard_irregular_plural() {
    // Dropping the `f -> ves` rule (so `chief` -> `chiefs` instead of `chieves`)
    // makes the rule predict `selfs`/`shelfs`/`leafs`, which are plainly attested
    // (the Toronto Maple `Leafs`, etc.) and would reserve the bare key. The
    // standard `-ves` plural must stay on the bare key.
    for (noun, expected) in [
        ("self", "selves"),
        ("shelf", "shelves"),
        ("leaf", "leaves"),
        ("sheaf", "sheaves"),
        ("hoof", "hooves"),
        ("scarf", "scarves"),
        ("wharf", "wharves"),
        ("dwarf", "dwarves"),
    ] {
        assert_eq!(plural(noun), expected, "plural of {noun}");
    }
}

#[test]
fn invariant_deer_stays_invariant() {
    // Dropping `deer -> deer` (so `balladeer` -> `balladeers`) makes the rule
    // predict `reindeers`, attested and would reserve the bare key over the
    // invariant `reindeer`.
    assert_eq!(plural("reindeer"), "reindeer");
    assert_eq!(plural("deer"), "deer");
}

#[test]
fn multisyllable_y_glide_adjectives_stay_periphrastic() {
    // `loyal`/`royal` are two syllables (o·yal) and take `more`/`most`, not
    // `loyaler`. The syllable heuristic counts `oya` as one vowel group, so the
    // table currently supplies the periphrastic form; keep it correct.
    assert_eq!(English::adj("loyal", &Degree::Comparative), "more loyal");
    assert_eq!(English::adj("loyal", &Degree::Superlative), "most loyal");
}
