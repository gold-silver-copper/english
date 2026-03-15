use english_phrase::*;

#[test]
fn small_phrase_inventory_is_recursive_through_boxed_phrase_nodes() {
    let inner = pp("near", dp(np("river")).the());
    let outer = pp("from", inner);

    assert!(matches!(outer.complement(), Phrase::PP(_)));
    assert_eq!(outer.realize().unwrap(), "from near the river");
}

#[test]
fn noun_phrases_and_determiner_phrases_are_distinct_public_nodes() {
    let noun = np("child")
        .modifier(adjp("happy").modifier(advp("very")))
        .complement(pp("with", dp(np("friend")).indefinite()));

    assert_eq!(
        noun.clone().realize().unwrap(),
        "very happy child with a friend"
    );
    assert_eq!(
        dp(noun).the().realize().unwrap(),
        "the very happy child with a friend"
    );
}

#[test]
fn indefinite_determiners_and_possessors_surface_from_dp() {
    assert_eq!(dp(np("friend")).indefinite().realize().unwrap(), "a friend");
    assert_eq!(dp(np("apple")).indefinite().realize().unwrap(), "an apple");
    assert_eq!(
        dp(np("book"))
            .possessor(dp(name("John")))
            .realize()
            .unwrap(),
        "John's book"
    );
}

#[test]
fn proper_names_and_pronouns_are_just_dp_heads() {
    assert_eq!(dp(name("Alice")).realize().unwrap(), "Alice");
    assert_eq!(dp(Pronoun::They).realize().unwrap(), "they");
}

#[test]
fn pronoun_forms_are_inferred_from_position_with_reflexive_override() {
    assert_eq!(
        tp(vp("see").present().complement(dp(Pronoun::She)))
            .subject(dp(Pronoun::He))
            .realize()
            .unwrap(),
        "he sees her"
    );
    assert_eq!(
        dp(np("book"))
            .possessor(dp(Pronoun::She))
            .realize()
            .unwrap(),
        "her book"
    );
    assert_eq!(
        tp(vp("admire")
            .present()
            .complement(dp(Pronoun::She).reflexive()))
        .subject(dp(Pronoun::She))
        .realize()
        .unwrap(),
        "she admires herself"
    );
}

#[test]
fn verb_phrases_handle_finite_and_non_finite_forms_without_extra_spine_nodes() {
    let infinitive = vp("eat")
        .to_infinitive()
        .negative()
        .complement(dp(np("apple")).the());

    let finite = vp("eat").past().complement(dp(np("apple")).the());

    assert_eq!(infinitive.realize().unwrap(), "not to eat the apple");
    assert_eq!(
        tp(finite.clone())
            .subject(dp(Pronoun::We))
            .realize()
            .unwrap(),
        "we ate the apple"
    );
    assert_eq!(
        tp(finite)
            .subject(dp(Pronoun::We))
            .sentence()
            .realize()
            .unwrap(),
        "We ate the apple."
    );
}

#[test]
fn adjective_and_adverb_phrases_reuse_the_same_recursive_phrase_enum() {
    let slowly = advp("slowly").complement(pp("along", dp(np("road")).the()));

    let careful = adjp("careful")
        .modifier(slowly.clone())
        .complement(pp("with", dp(name("Alice"))));

    assert_eq!(slowly.realize().unwrap(), "slowly along the road");
    assert_eq!(
        careful.realize().unwrap(),
        "slowly along the road careful with Alice"
    );
}
