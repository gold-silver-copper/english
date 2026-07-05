//! Exhaustive coverage of the suppletive verb `be`.
//!
//! `English::verb("be", ...)` routes through `EnglishCore::to_be`, a 12-cell
//! suppletive table — the only English verb whose finite forms vary by person AND
//! number. Before this test the sole assertion touching it was `be/1sg/present ->
//! "am"` in an example, so swapping the singular arms ("I were") would pass CI.
//! This pins every reachable cell.

use english::*;

fn be(person: Person, number: Number, tense: Tense, form: Form) -> String {
    English::verb("be", &person, &number, &tense, &form)
}

#[test]
fn present_finite_varies_by_person_and_number() {
    assert_eq!(
        be(
            Person::First,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "am"
    );
    assert_eq!(
        be(
            Person::Second,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "are"
    );
    assert_eq!(
        be(
            Person::Third,
            Number::Singular,
            Tense::Present,
            Form::Finite
        ),
        "is"
    );
    // Every plural person is "are".
    assert_eq!(
        be(Person::First, Number::Plural, Tense::Present, Form::Finite),
        "are"
    );
    assert_eq!(
        be(Person::Second, Number::Plural, Tense::Present, Form::Finite),
        "are"
    );
    assert_eq!(
        be(Person::Third, Number::Plural, Tense::Present, Form::Finite),
        "are"
    );
}

#[test]
fn past_finite_varies_by_person_and_number() {
    assert_eq!(
        be(Person::First, Number::Singular, Tense::Past, Form::Finite),
        "was"
    );
    assert_eq!(
        be(Person::Second, Number::Singular, Tense::Past, Form::Finite),
        "were"
    );
    assert_eq!(
        be(Person::Third, Number::Singular, Tense::Past, Form::Finite),
        "was"
    );
    // Every plural person is "were".
    assert_eq!(
        be(Person::First, Number::Plural, Tense::Past, Form::Finite),
        "were"
    );
    assert_eq!(
        be(Person::Second, Number::Plural, Tense::Past, Form::Finite),
        "were"
    );
    assert_eq!(
        be(Person::Third, Number::Plural, Tense::Past, Form::Finite),
        "were"
    );
}

#[test]
fn participles_and_infinitive_are_invariant() {
    for person in [Person::First, Person::Second, Person::Third] {
        for number in [Number::Singular, Number::Plural] {
            assert_eq!(
                be(person, number, Tense::Present, Form::Participle),
                "being"
            );
            assert_eq!(be(person, number, Tense::Past, Form::Participle), "been");
            // Infinitive ignores tense entirely.
            assert_eq!(be(person, number, Tense::Present, Form::Infinitive), "be");
            assert_eq!(be(person, number, Tense::Past, Form::Infinitive), "be");
        }
    }
}
