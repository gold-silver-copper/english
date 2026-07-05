//! Runtime tests for `English::adverb`: the periphrastic rule fallback, the
//! table-backed flat/suppletive adverbs, case handling, and sense-numbered keys.

use english::{Degree, English};

fn comp(word: &str) -> String {
    English::adverb(word, &Degree::Comparative)
}
fn sup(word: &str) -> String {
    English::adverb(word, &Degree::Superlative)
}

#[test]
fn positive_returns_the_base_adverb() {
    assert_eq!(English::adverb("quickly", &Degree::Positive), "quickly");
    assert_eq!(English::adverb("well", &Degree::Positive), "well");
    assert_eq!(English::adverb("fast", &Degree::Positive), "fast");
}

#[test]
fn regular_adverbs_are_periphrastic_by_rule() {
    // The overwhelming majority of gradable adverbs (the `-ly` class) take
    // more/most and are served by the rule with no table row.
    assert_eq!(comp("quickly"), "more quickly");
    assert_eq!(sup("quickly"), "most quickly");
    assert_eq!(comp("abruptly"), "more abruptly");
    assert_eq!(sup("carefully"), "most carefully");
    assert_eq!(comp("happily"), "more happily");
}

#[test]
fn suppletive_adverbs_come_from_the_table() {
    assert_eq!(comp("well"), "better");
    assert_eq!(sup("well"), "best");
    assert_eq!(comp("badly"), "worse");
    assert_eq!(sup("badly"), "worst");
    assert_eq!(comp("good"), "better"); // the flat adverb "good" (informal), attested
    assert_eq!(sup("good"), "best");
}

#[test]
fn flat_adverbs_take_suffixal_forms_from_the_table() {
    // Flat adverbs are homographs of their adjectives and inflect suffixally;
    // they are table-driven because the conservative rule is periphrastic.
    for (adv, c, s) in [
        ("fast", "faster", "fastest"),
        ("early", "earlier", "earliest"),
        ("hard", "harder", "hardest"),
        ("late", "later", "latest"),
        ("soon", "sooner", "soonest"),
        ("near", "nearer", "nearest"),
        ("loud", "louder", "loudest"),
        ("long", "longer", "longest"),
        ("high", "higher", "highest"),
    ] {
        assert_eq!(comp(adv), c, "comparative of {adv}");
        assert_eq!(sup(adv), s, "superlative of {adv}");
    }
}

#[test]
fn far_homograph_is_split_into_numbered_keys() {
    // `far` attests both farther/farthest and further/furthest; the bare key holds
    // the form-signature-first pair and the other is a numbered key.
    assert_eq!(comp("far"), "farther");
    assert_eq!(sup("far"), "farthest");
    assert_eq!(comp("far_2"), "further");
    assert_eq!(sup("far_2"), "furthest");
}

#[test]
fn periphrastic_adverb_with_an_attested_suffixal_variant_keeps_periphrastic_bare() {
    // POLICY (deliberate): when an adverb attests BOTH "more X" and a single-word
    // form, the periphrastic wins the bare key (prefer_regular = true) and the
    // single-word lands at a numbered key. This is correct for the large -ly class
    // (quickly's archaic "quicklier"), and both forms stay reachable. For the few
    // flat adverbs that also attest "more X" (deep), the suffixal is a numbered
    // key — a documented casualty, not an accuracy loss (deeper is reachable).
    assert_eq!(comp("quickly"), "more quickly");
    assert_eq!(comp("quickly_2"), "quicklier");
    assert_eq!(sup("quickly_2"), "quickliest");
    // deep attests both; "more deep" is the bare key, "deeper" the numbered key.
    assert_eq!(comp("deep"), "more deep");
    assert_eq!(comp("deep_2"), "deeper");
    assert_eq!(sup("deep_2"), "deepest");
}

#[test]
fn case_is_restored_like_the_other_pos_apis() {
    // Table hits restore casing onto the value; rule fallbacks get the same
    // treatment via rule_with_case — exactly as adjectives (HOT -> HOTTER).
    assert_eq!(comp("WELL"), "BETTER");
    assert_eq!(sup("WELL"), "BEST");
    assert_eq!(comp("FAST"), "FASTER");
    assert_eq!(comp("Fast"), "Faster");
    // Rule (periphrastic) fallback preserves/restores case too.
    assert_eq!(comp("QUICKLY"), "MORE QUICKLY");
    assert_eq!(comp("Quickly"), "More quickly");
    assert_eq!(sup("Quickly"), "Most quickly");
    // Mixed case is not guessed at: table misses, rule applies on the input.
    assert_eq!(comp("McFast"), "more McFast");
}

#[test]
fn sense_suffix_resolves_through_the_base_lemma() {
    // A `_<n>` suffix on a bare irregular key resolves to that base lemma.
    assert_eq!(English::adverb("far_2", &Degree::Positive), "far");
    assert_eq!(comp("well_7"), "better"); // suffixed alias of the bare key `well`
    assert_eq!(English::adverb("well_7", &Degree::Positive), "well");
}

#[test]
fn non_key_suffixes_and_digits_stay_opaque() {
    // A `_<n>` suffix that resolves to no adverb key is opaque; the periphrastic
    // rule then runs on the whole string.
    assert_eq!(comp("cat_2"), "more cat_2");
    // Digit-bearing non-keys are never mangled into a stripped stem.
    assert_eq!(comp("mp3"), "more mp3");
    assert_eq!(English::adverb("mp3", &Degree::Positive), "mp3");
}
