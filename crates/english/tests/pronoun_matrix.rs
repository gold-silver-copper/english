//! Exhaustive coverage of the `English::pronoun` matrix.
//!
//! `EnglishCore::pronoun` is a ~40-cell hand-written match. Before this test the
//! only assertions were the two 1st-person-singular cells in `examples/test.rs`,
//! so a transposed cell (`hers`/`her`, `ours`/`our`, `theirs`/`their`) or a wrong
//! reflexive could ship silently. This pins every cell.

use english::*;

fn p(person: Person, number: Number, gender: Gender, case: Case) -> &'static str {
    English::pronoun(&person, &number, &gender, &case)
}

// Gender is only consulted for the 3rd-person singular; for every other row any
// gender must yield the same pronoun.
const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];

#[test]
fn first_person_singular_ignores_gender() {
    for g in GENDERS {
        assert_eq!(p(Person::First, Number::Singular, g, Case::Nominative), "I");
        assert_eq!(
            p(Person::First, Number::Singular, g, Case::Accusative),
            "me"
        );
        assert_eq!(
            p(Person::First, Number::Singular, g, Case::Reflexive),
            "myself"
        );
        assert_eq!(
            p(Person::First, Number::Singular, g, Case::Possessive),
            "mine"
        );
        assert_eq!(
            p(Person::First, Number::Singular, g, Case::PersonalPossessive),
            "my"
        );
    }
}

#[test]
fn second_person_singular_ignores_gender() {
    for g in GENDERS {
        assert_eq!(
            p(Person::Second, Number::Singular, g, Case::Nominative),
            "you"
        );
        assert_eq!(
            p(Person::Second, Number::Singular, g, Case::Accusative),
            "you"
        );
        assert_eq!(
            p(Person::Second, Number::Singular, g, Case::Reflexive),
            "yourself"
        );
        assert_eq!(
            p(Person::Second, Number::Singular, g, Case::Possessive),
            "yours"
        );
        assert_eq!(
            p(
                Person::Second,
                Number::Singular,
                g,
                Case::PersonalPossessive
            ),
            "your"
        );
    }
}

#[test]
fn third_person_singular_is_gendered() {
    // Masculine.
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            Case::Nominative
        ),
        "he"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            Case::Accusative
        ),
        "him"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            Case::Reflexive
        ),
        "himself"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            Case::Possessive
        ),
        "his"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            Case::PersonalPossessive
        ),
        "his"
    );
    // Feminine — the possessive/personal-possessive split (hers vs her) is easy to transpose.
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Feminine,
            Case::Nominative
        ),
        "she"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Feminine,
            Case::Accusative
        ),
        "her"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Feminine,
            Case::Reflexive
        ),
        "herself"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Feminine,
            Case::Possessive
        ),
        "hers"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Feminine,
            Case::PersonalPossessive
        ),
        "her"
    );
    // Neuter.
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Neuter,
            Case::Nominative
        ),
        "it"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Neuter,
            Case::Accusative
        ),
        "it"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Neuter,
            Case::Reflexive
        ),
        "itself"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Neuter,
            Case::Possessive
        ),
        "its"
    );
    assert_eq!(
        p(
            Person::Third,
            Number::Singular,
            Gender::Neuter,
            Case::PersonalPossessive
        ),
        "its"
    );
}

#[test]
fn first_person_plural_ignores_gender() {
    for g in GENDERS {
        assert_eq!(p(Person::First, Number::Plural, g, Case::Nominative), "we");
        assert_eq!(p(Person::First, Number::Plural, g, Case::Accusative), "us");
        assert_eq!(
            p(Person::First, Number::Plural, g, Case::Reflexive),
            "ourselves"
        );
        assert_eq!(
            p(Person::First, Number::Plural, g, Case::Possessive),
            "ours"
        );
        assert_eq!(
            p(Person::First, Number::Plural, g, Case::PersonalPossessive),
            "our"
        );
    }
}

#[test]
fn second_person_plural_ignores_gender() {
    for g in GENDERS {
        assert_eq!(
            p(Person::Second, Number::Plural, g, Case::Nominative),
            "you"
        );
        assert_eq!(
            p(Person::Second, Number::Plural, g, Case::Accusative),
            "you"
        );
        assert_eq!(
            p(Person::Second, Number::Plural, g, Case::Reflexive),
            "yourselves"
        );
        assert_eq!(
            p(Person::Second, Number::Plural, g, Case::Possessive),
            "yours"
        );
        assert_eq!(
            p(Person::Second, Number::Plural, g, Case::PersonalPossessive),
            "your"
        );
    }
}

#[test]
fn third_person_plural_ignores_gender() {
    for g in GENDERS {
        assert_eq!(
            p(Person::Third, Number::Plural, g, Case::Nominative),
            "they"
        );
        assert_eq!(
            p(Person::Third, Number::Plural, g, Case::Accusative),
            "them"
        );
        assert_eq!(
            p(Person::Third, Number::Plural, g, Case::Reflexive),
            "themselves"
        );
        assert_eq!(
            p(Person::Third, Number::Plural, g, Case::Possessive),
            "theirs"
        );
        assert_eq!(
            p(Person::Third, Number::Plural, g, Case::PersonalPossessive),
            "their"
        );
    }
}
