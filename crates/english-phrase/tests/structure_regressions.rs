use english_phrase::*;

#[test]
fn phrase_enum_includes_public_tp() {
    let phrase: Phrase = tp(vp("admire").complement(dp(Pronoun::She)))
        .present()
        .subject(dp(Pronoun::He))
        .into();

    assert!(matches!(phrase, Phrase::TP(_)));
}

#[test]
fn phrase_enum_includes_public_cp() {
    let phrase: Phrase = cp(tp(vp("admire").complement(dp(Pronoun::She)))
        .past()
        .subject(dp(Pronoun::He)))
    .that()
    .into();

    assert!(matches!(phrase, Phrase::CP(_)));
}

#[test]
fn bare_vp_realizes_as_a_lexical_projection_not_a_clause() {
    assert_eq!(
        vp("admire")
            .complement(dp(Pronoun::She))
            .adjunct(advp("openly"))
            .realize()
            .unwrap(),
        "admire her openly"
    );
}

#[test]
fn pp_complements_use_object_case_for_pronouns() {
    assert_eq!(pp("with", dp(Pronoun::She)).realize().unwrap(), "with her");
}

#[test]
fn orthography_is_realization_not_syntax() {
    let clause = tp(vp("admire").complement(dp(Pronoun::She)))
        .present()
        .subject(dp(Pronoun::He));

    assert_eq!(clause.realize().unwrap(), "he admires her");
    assert_eq!(
        clause.realize_with(RealizationOptions::sentence()).unwrap(),
        "He admires her."
    );
}

#[test]
fn overt_complementizers_realize_above_tp() {
    let phrase = cp(tp(vp("admire").complement(dp(Pronoun::She)))
        .past()
        .subject(dp(name("Alice"))))
    .that();

    assert_eq!(phrase.realize().unwrap(), "that Alice admired her");
}

#[test]
fn noun_phrase_clause_complements_reject_bare_infinitives() {
    let phrase = np("attempt").complement(tp(vp("leave")).bare_infinitive());

    assert!(phrase.realize().is_err());
}

#[test]
fn prepositional_clause_complements_reject_to_infinitives() {
    let phrase = pp("after", tp(vp("leave")).to_infinitive());

    assert!(phrase.realize().is_err());
}

#[test]
fn noun_phrases_distinguish_selected_complements_from_pp_adjuncts() {
    let phrase = np("map")
        .complement(pp("of", dp(np("cave")).the()))
        .adjunct(pp("from", dp(np("museum")).the()));

    assert_eq!(phrase.realize().unwrap(), "map of the cave from the museum");
}
