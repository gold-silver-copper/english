use english_phrase::*;

#[test]
fn finite_clauses_remain_public_values() {
    let clause = tp(vp("admire").complement(dp(Pronoun::She)))
        .present()
        .subject(dp(Pronoun::He));

    assert_eq!(clause.realize(), "he admires her");
}

#[test]
fn complementizer_phrases_remain_public_values() {
    let phrase = cp(tp(vp("admire").complement(dp(Pronoun::She)))
        .past()
        .subject(dp(Pronoun::He)))
    .that();

    assert_eq!(phrase.realize(), "that he admired her");
}

#[test]
fn bare_vp_realizes_as_a_lexical_projection_not_a_clause() {
    assert_eq!(
        vp("admire")
            .complement(dp(Pronoun::She))
            .adjunct(advp("openly"))
            .realize(),
        "admire her openly"
    );
}

#[test]
fn pp_complements_use_object_case_for_pronouns() {
    assert_eq!(pp("with", dp(Pronoun::She)).realize(), "with her");
}

#[test]
fn phrase_pronouns_delegate_to_the_feature_based_paradigm() {
    assert_eq!(dp(Pronoun::YouPlural).reflexive().realize(), "yourselves");
    assert_eq!(
        pp(
            "with",
            dp(Pronoun::new(
                Person::Third,
                Number::Singular,
                Gender::Masculine,
            )),
        )
        .realize(),
        "with him"
    );
}

#[test]
fn orthography_is_realization_not_syntax() {
    let clause = tp(vp("admire").complement(dp(Pronoun::She)))
        .present()
        .subject(dp(Pronoun::He));

    assert_eq!(clause.realize(), "he admires her");
    assert_eq!(
        clause.realize_with(RealizationOptions::sentence()),
        "He admires her."
    );
}

#[test]
fn overt_complementizers_realize_above_tp() {
    let phrase = cp(tp(vp("admire").complement(dp(Pronoun::She)))
        .past()
        .subject(dp(name("Alice"))))
    .that();

    assert_eq!(phrase.realize(), "that Alice admired her");
}

#[test]
fn noun_phrase_clause_complements_accept_to_infinitives() {
    let phrase = np("attempt").complement(tp(vp("leave")).to_infinitive());

    assert_eq!(phrase.realize(), "attempt to leave");
}

#[test]
fn prepositional_clause_complements_accept_gerunds() {
    let phrase = pp("after", tp(vp("leave")).gerund_participle());

    assert_eq!(phrase.realize(), "after leaving");
}

#[test]
fn noun_phrases_distinguish_selected_complements_from_pp_adjuncts() {
    let phrase = np("map")
        .complement(pp("of", dp(np("cave")).the()))
        .adjunct(pp("from", dp(np("museum")).the()));

    assert_eq!(phrase.realize(), "map of the cave from the museum");
}

#[test]
fn object_gap_relative_clauses_realize_as_true_filler_gap_dependencies() {
    let phrase = np("editor").relative(
        cp(
            tp(vp("admire").object_gap())
                .past()
                .subject(dp(name("Alice"))),
        )
        .that(),
    );

    assert_eq!(phrase.realize(), "editor that Alice admired");
}

#[test]
fn subject_gap_relative_clauses_carry_number_agreement_from_their_type() {
    let singular = np("editor").relative(
        cp(
            tp(vp("admire").complement(dp(Pronoun::She)))
                .present()
                .subject_gap::<SingularNumber>(),
        )
        .who(),
    );
    let plural = np("editor").plural().relative(
        cp(
            tp(vp("admire").complement(dp(Pronoun::She)))
                .present()
                .subject_gap::<PluralNumber>(),
        )
        .who(),
    );

    assert_eq!(singular.realize(), "editor who admires her");
    assert_eq!(plural.realize(), "editors who admire her");
}
