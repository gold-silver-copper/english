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
        tp(vp("see").complement(dp(Pronoun::She)))
            .present()
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
        tp(vp("admire").complement(dp(Pronoun::She).reflexive()))
            .present()
            .subject(dp(Pronoun::She))
            .realize()
            .unwrap(),
        "she admires herself"
    );
}

#[test]
fn verb_phrases_stay_lexical_and_tense_phrases_carry_inflection() {
    let lexical = vp("eat").complement(dp(np("apple")).the());
    let infinitive = tp(lexical.clone()).to_infinitive().negative();
    let finite = tp(lexical.clone()).past();

    assert_eq!(lexical.realize().unwrap(), "eat the apple");
    assert_eq!(infinitive.realize().unwrap(), "not to eat the apple");
    assert_eq!(
        finite.clone().subject(dp(Pronoun::We)).realize().unwrap(),
        "we ate the apple"
    );
    assert_eq!(
        finite
            .subject(dp(Pronoun::We))
            .realize_with(RealizationOptions::sentence())
            .unwrap(),
        "We ate the apple."
    );
}

#[test]
fn pp_complements_render_pronouns_in_object_case() {
    assert_eq!(pp("with", dp(Pronoun::She)).realize().unwrap(), "with her");
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
