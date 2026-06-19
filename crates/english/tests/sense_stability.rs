//! Golden regression tests pinning the meaning of published, sense-numbered keys.
//!
//! These lock in the public contract that the assignment lockfiles exist to
//! protect: a key like `lie2` or `die2` must keep producing the same inflection
//! across releases. If a data refresh ever renumbers a homograph, one of these
//! fails loudly instead of silently shipping a swapped meaning.

use english::{Degree, English, Form, Number, Person, Tense};

fn past_finite(word: &str) -> String {
    English::verb(
        word,
        &Person::Third,
        &Number::Singular,
        &Tense::Past,
        &Form::Finite,
    )
}

#[test]
fn verb_lie_homographs_are_pinned() {
    // "lie" (recline) -> lay ; "lie2" (tell an untruth) -> lied
    assert_eq!(past_finite("lie"), "lay");
    assert_eq!(past_finite("lie2"), "lied");
}

#[test]
fn noun_die_homograph_is_pinned() {
    assert_eq!(English::noun("die2", &Number::Plural), "dice");
}

#[test]
fn adjective_bad_homographs_are_pinned() {
    assert_eq!(English::adj("bad2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad2", &Degree::Superlative), "baddest");
    assert_eq!(English::adj("bad3", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad3", &Degree::Superlative), "worst");
}

#[test]
fn unknown_suffixed_key_strips_all_digits_and_falls_back_to_rules() {
    // Keys not in the tables must strip their (possibly multi-digit) suffix and
    // conjugate/pluralize regularly — guards the strip-all-trailing-digits fix.
    assert_eq!(English::noun("cat2", &Number::Plural), "cats");
    assert_eq!(English::noun("cat12", &Number::Plural), "cats");
    assert_eq!(past_finite("walk2"), "walked");
    assert_eq!(past_finite("walk99"), "walked");
}
