use english_phrase::*;

#[test]
fn small_phrase_inventory_is_recursive_through_boxed_phrase_nodes() {
    let inner = pp("near", dp("river").determiner(Determiner::The));
    let outer = pp("from", inner);

    assert!(matches!(outer.complement(), Phrase::PP(_)));
    assert_eq!(
        realize_prepositional_phrase(outer).unwrap(),
        "from near the river"
    );
}

#[test]
fn determiner_phrases_are_built_through_functional_combinators() {
    let child = dp("child")
        .determiner(Determiner::The)
        .modifier(adjp("happy").modifier(advp("very")))
        .complement(pp("with", dp("friend").determiner(Determiner::A)));

    assert_eq!(
        realize_determiner_phrase(child).unwrap(),
        "the very happy child with a friend"
    );
}

#[test]
fn proper_names_and_pronouns_are_just_dp_heads() {
    assert_eq!(
        realize_determiner_phrase(proper_name("Alice")).unwrap(),
        "Alice"
    );
    assert_eq!(
        realize_determiner_phrase(pronoun_dp(Pronoun::They)).unwrap(),
        "they"
    );
}

#[test]
fn verb_phrases_handle_finite_and_non_finite_forms_without_extra_spine_nodes() {
    let infinitive = vp("eat")
        .to_infinitive()
        .negative()
        .complement(dp("apple").determiner(Determiner::The));

    let finite = vp("eat")
        .past()
        .complement(dp("apple").determiner(Determiner::The));

    assert_eq!(
        realize_verb_phrase(infinitive).unwrap(),
        "not to eat the apple"
    );
    assert_eq!(
        realize_clause(pronoun_dp(Pronoun::We), finite.clone()).unwrap(),
        "we ate the apple"
    );
    assert_eq!(
        realize_sentence(pronoun_dp(Pronoun::We), finite).unwrap(),
        "We ate the apple."
    );
}

#[test]
fn adjective_and_adverb_phrases_reuse_the_same_recursive_phrase_enum() {
    let slowly = advp("slowly").complement(pp("along", dp("road").determiner(Determiner::The)));

    let careful = adjp("careful")
        .modifier(slowly.clone())
        .complement(pp("with", proper_name("Alice")));

    assert_eq!(
        realize_adverb_phrase(slowly).unwrap(),
        "slowly along the road"
    );
    assert_eq!(
        realize_adjective_phrase(careful).unwrap(),
        "slowly along the road careful with Alice"
    );
}
