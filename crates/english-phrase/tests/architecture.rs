use english_phrase::*;

#[test]
fn small_phrase_inventory_is_recursive_through_boxed_phrase_nodes() {
    let mut river = dp("river");
    river.determiner = Some(Determiner::The);

    let inner = pp("near", river);
    let outer = pp("from", inner);

    assert!(matches!(outer.complement.as_ref(), Phrase::PP(_)));
    assert_eq!(
        realize_prepositional_phrase(&outer).unwrap(),
        "from near the river"
    );
}

#[test]
fn determiner_phrases_are_direct_structs_without_builders() {
    let mut friend = dp("friend");
    friend.determiner = Some(Determiner::A);

    let mut modifier = adjp("happy");
    modifier.modifier = Some(Box::new(advp("very").into()));

    let mut child = dp("child");
    child.determiner = Some(Determiner::The);
    child.modifiers.push(Box::new(modifier.into()));
    child.complements.push(Box::new(pp("with", friend).into()));

    assert_eq!(
        realize_determiner_phrase(&child).unwrap(),
        "the very happy child with a friend"
    );
}

#[test]
fn proper_names_and_pronouns_are_just_dp_heads() {
    let alice = proper_name("Alice");
    let they = pronoun_dp(Pronoun::They);

    assert_eq!(realize_determiner_phrase(&alice).unwrap(), "Alice");
    assert_eq!(realize_determiner_phrase(&they).unwrap(), "they");
}

#[test]
fn verb_phrases_handle_finite_and_non_finite_forms_without_extra_spine_nodes() {
    let mut object = dp("apple");
    object.determiner = Some(Determiner::The);

    let mut infinitive = vp("eat");
    infinitive.form = VerbForm::ToInfinitive;
    infinitive.negative = true;
    infinitive.complements.push(Box::new(object.clone().into()));

    let mut finite = VerbPhrase::finite("eat", Tense::Past);
    finite.complements.push(Box::new(object.into()));

    let subject = pronoun_dp(Pronoun::We);

    assert_eq!(
        realize_verb_phrase(&infinitive).unwrap(),
        "not to eat the apple"
    );
    assert_eq!(
        realize_clause(&subject, &finite).unwrap(),
        "we ate the apple"
    );
    assert_eq!(
        realize_sentence(&subject, &finite).unwrap(),
        "We ate the apple."
    );
}

#[test]
fn adjective_and_adverb_phrases_reuse_the_same_recursive_phrase_enum() {
    let mut road = dp("road");
    road.determiner = Some(Determiner::The);

    let mut slowly = advp("slowly");
    slowly.complements.push(Box::new(pp("along", road).into()));

    let mut careful = adjp("careful");
    careful.modifier = Some(Box::new(slowly.clone().into()));
    careful
        .complements
        .push(Box::new(pp("with", proper_name("Alice")).into()));

    assert_eq!(
        realize_adverb_phrase(&slowly).unwrap(),
        "slowly along the road"
    );
    assert_eq!(
        realize_adjective_phrase(&careful).unwrap(),
        "slowly along the road careful with Alice"
    );
}
