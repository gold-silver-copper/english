//! Coverage for two under-tested surfaces:
//!  1. the irregular-verb non-3sg present-finite arm (returns the bare lemma), and
//!  2. the `count`/`count_with_number`/`add_possessive` surface helpers.

use english::*;

/// A TABLE-backed irregular verb: the present-finite arm returns the bare lemma
/// for every person/number EXCEPT 3rd-singular, which takes the tabled form. A
/// regression that let the non-3sg arm fall through to the 3sg slot would give
/// "I goes" — this pins it. (The bug the arm guards against is `lib.rs`'s
/// `(_, _, Present, Finite) => base_word`.)
#[test]
fn irregular_verb_present_finite_is_the_bare_lemma_except_3sg() {
    let g = |person, number, tense, form| English::verb("go", &person, &number, &tense, &form);
    // Non-3sg present finite -> bare lemma.
    assert_eq!(
        g(
            Person::First,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "go"
    );
    assert_eq!(
        g(
            Person::Second,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "go"
    );
    assert_eq!(
        g(Person::First, Number::Plural, Tense::Present, Form::Finite),
        "go"
    );
    assert_eq!(
        g(Person::Third, Number::Plural, Tense::Present, Form::Finite),
        "go"
    );
    // 3sg present finite -> the tabled form.
    assert_eq!(
        g(
            Person::Third,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "goes"
    );
    // The other slots still come from the table.
    assert_eq!(
        g(Person::First, Number::Singular, Tense::Past, Form::Finite),
        "went"
    );
    assert_eq!(
        g(
            Person::Third,
            Number::Singular,
            Tense::Past,
            Form::Participle
        ),
        "gone"
    );
    assert_eq!(
        g(
            Person::First,
            Number::Singular,
            Tense::Present,
            Form::Participle
        ),
        "going"
    );
    assert_eq!(
        g(
            Person::First,
            Number::Singular,
            Tense::Present,
            Form::Infinitive
        ),
        "go"
    );
}

#[test]
fn regular_verb_present_finite_is_the_bare_lemma_except_3sg() {
    let w = |person, number, tense, form| English::verb("walk", &person, &number, &tense, &form);
    assert_eq!(
        w(
            Person::First,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "walk"
    );
    assert_eq!(
        w(Person::Second, Number::Plural, Tense::Present, Form::Finite),
        "walk"
    );
    assert_eq!(
        w(
            Person::Third,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "walks"
    );
    assert_eq!(
        w(Person::Third, Number::Plural, Tense::Past, Form::Finite),
        "walked"
    );
}

#[test]
fn count_selects_singular_only_for_exactly_one() {
    // 0 and >1 take the plural; 1 takes the singular.
    assert_eq!(count("cat", 0), "cats");
    assert_eq!(count("cat", 1), "cat");
    assert_eq!(count("cat", 2), "cats");
    assert_eq!(count("cat", 100), "cats");
    // Irregular / spelling-rule plurals flow through.
    assert_eq!(count("child", 1), "child");
    assert_eq!(count("child", 3), "children");
    assert_eq!(count("box", 0), "boxes");
}

#[test]
fn count_with_number_keeps_the_number() {
    assert_eq!(count_with_number("cat", 0), "0 cats");
    assert_eq!(count_with_number("cat", 1), "1 cat");
    assert_eq!(count_with_number("cat", 2), "2 cats");
    assert_eq!(count_with_number("cat", 100), "100 cats");
}

#[test]
fn add_possessive_uses_apostrophe_only_after_a_trailing_s() {
    // Non-s words take 's; plurals ending in s take a bare apostrophe.
    assert_eq!(English::add_possessive("dog"), "dog's");
    assert_eq!(English::add_possessive("dogs"), "dogs'");
    assert_eq!(English::add_possessive("child"), "child's");
    assert_eq!(English::add_possessive("children"), "children's");
    assert_eq!(English::add_possessive("box"), "box's");
    assert_eq!(English::add_possessive("boxes"), "boxes'");

    // KNOWN LIMITATION (audit B2): the rule treats ANY trailing `s` as a plural,
    // so a SINGULAR noun ending in `s` gets a bare apostrophe ("bus'") where
    // standard English wants "bus's". This is a characterization test pinning the
    // CURRENT behavior; if `add_possessive` is taught to distinguish singular vs
    // plural, update these expectations deliberately.
    assert_eq!(English::add_possessive("bus"), "bus'");
    assert_eq!(English::add_possessive("boss"), "boss'");
}
