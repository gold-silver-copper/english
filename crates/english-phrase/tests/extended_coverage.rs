use english_phrase::*;

fn dp(noun: &str) -> DeterminerPhrase {
    DeterminerPhrase::new(noun)
}

fn ap(adjective: &str) -> AdjPhrase {
    AdjPhrase::new(adjective)
}

fn vp(verb: &str) -> VerbPhrase {
    VerbPhrase::new(verb)
}

fn pp(preposition: &str, complement: DeterminerPhrase) -> PrepositionalPhrase {
    PrepositionalPhrase::new(preposition, complement)
}

#[test]
fn determiners_render_all_built_in_variants() {
    assert_eq!(dp("child").determiner(Determiner::the()).render(), "the child");
    assert_eq!(dp("child").determiner(Determiner::a()).render(), "a child");
    assert_eq!(dp("apple").determiner(Determiner::an()).render(), "an apple");
    assert_eq!(dp("child").determiner(Determiner::this()).render(), "this child");
    assert_eq!(dp("child").determiner(Determiner::that()).render(), "that child");
    assert_eq!(dp("child").plural().determiner(Determiner::these()).render(), "these children");
    assert_eq!(dp("child").plural().determiner(Determiner::those()).render(), "those children");
    assert_eq!(
        dp("child")
            .determiner(Determiner::custom("each"))
            .render(),
        "each child"
    );
}

#[test]
fn custom_determiners_modifiers_and_particles_render_cleanly() {
    assert_eq!(
        dp("child")
            .determiner(Determiner::custom("every"))
            .modifier(ap("sleepy"))
            .render(),
        "every sleepy child"
    );
    assert_eq!(
        vp("give")
            .particle("away")
            .past()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "gave away"
    );
}

#[test]
fn public_prepositional_phrases_render_with_noun_phrase_complements() {
    assert_eq!(
        pp("on", dp("wall").the()).render(),
        "on the wall"
    );
    assert_eq!(
        pp("near", dp("station").the()).render(),
        "near the station"
    );
}

#[test]
fn adverb_phrases_support_specifiers_and_prepositional_complements() {
    assert_eq!(
        AdverbPhrase::new("far")
            .specifier(AdverbPhrase::new("very"))
            .render(),
        "very far"
    );
    assert_eq!(
        AdverbPhrase::new("far")
            .specifier(AdverbPhrase::new("quite").specifier(AdverbPhrase::new("very")))
            .render(),
        "very quite far"
    );
    assert_eq!(
        AdverbPhrase::new("independently")
            .complement(pp("of", dp("help")))
            .render(),
        "independently of help"
    );
}

#[test]
fn adjective_positive_comparative_and_superlative_render() {
    assert_eq!(AdjPhrase::new("small").positive().render(), "small");
    assert_eq!(AdjPhrase::new("small").comparative().render(), "smaller");
    assert_eq!(AdjPhrase::new("small").superlative().render(), "smallest");
    assert_eq!(AdjPhrase::new("bad3").positive().render(), "bad");
    assert_eq!(AdjPhrase::new("bad3").comparative().render(), "worse");
    assert_eq!(AdjPhrase::new("bad3").superlative().render(), "worst");
}

#[test]
fn adjective_intensifiers_stack_with_degrees() {
    assert_eq!(
        AdjPhrase::new("small")
            .comparative()
            .intensifier(AdverbPhrase::new("much"))
            .render(),
        "much smaller"
    );
    assert_eq!(
        AdjPhrase::new("bad3")
            .superlative()
            .intensifier(AdverbPhrase::new("far").specifier(AdverbPhrase::new("by")))
            .render(),
        "by far worst"
    );
    assert_eq!(
        AdjPhrase::new("fun")
            .comparative()
            .intensifier(AdverbPhrase::new("much"))
            .render(),
        "much more fun"
    );
}

#[test]
fn adjective_phrase_supports_prepositional_complements() {
    assert_eq!(
        AdjPhrase::new("full")
            .positive()
            .complement(pp("of", dp("bean").plural()))
            .render(),
        "full of beans"
    );
    assert_eq!(
        AdjPhrase::new("close")
            .positive()
            .complement(pp("to", dp("station").the()))
            .render(),
        "close to the station"
    );
}

#[test]
fn noun_phrase_singular_plural_and_counts_render() {
    assert_eq!(dp("child").render(), "child");
    assert_eq!(dp("child").plural().render(), "children");
    assert_eq!(dp("child").count(0).render(), "0 children");
    assert_eq!(dp("child").count(1).render(), "1 child");
    assert_eq!(dp("child").count(2).render(), "2 children");
    assert_eq!(dp("potato").count(7).render(), "7 potatoes");
}

#[test]
fn noun_phrase_renders_adjective_modifiers_before_the_head() {
    assert_eq!(
        dp("child")
            .modifier(ap("running"))
            .render(),
        "running child"
    );
    assert_eq!(
        dp("child")
            .plural()
            .modifier(ap("running"))
            .modifier(ap("hungry"))
            .render(),
        "running hungry children"
    );
    assert_eq!(
        dp("child")
            .count(3)
            .modifier(ap("running"))
            .modifier(ap("hungry"))
            .render(),
        "3 running hungry children"
    );
}

#[test]
fn noun_phrase_renders_adjective_phrase_modifiers() {
    assert_eq!(
        dp("child")
            .modifier(AdjPhrase::new("small").positive())
            .render(),
        "small child"
    );
    assert_eq!(
        dp("child")
            .plural()
            .determiner(Determiner::the())
            .modifier(AdjPhrase::new("small").comparative())
            .render(),
        "the smaller children"
    );
    assert_eq!(
        dp("day")
            .determiner(Determiner::the())
            .modifier(AdjPhrase::new("bad3").superlative())
            .render(),
        "the worst day"
    );
}

#[test]
fn noun_phrase_supports_structured_prepositional_complements() {
    assert_eq!(
        dp("pair")
            .count(3)
            .complement(pp("of", dp("jean").plural()))
            .render(),
        "3 pairs of jeans"
    );
    assert_eq!(
        dp("door")
            .the()
            .complement(pp("of", dp("house").the()))
            .render(),
        "the door of the house"
    );
}

#[test]
fn noun_phrase_supports_deep_recursive_boxed_complements() {
    let phrase = dp("photo")
        .the()
        .complement(pp(
            "of",
            dp("child")
                .the()
                .postmodifier(pp(
                    "with",
                    dp("toy")
                        .the()
                        .postmodifier(pp("in", dp("box").the())),
                )),
        ));

    assert_eq!(phrase.render(), "the photo of the child with the toy in the box");
}

#[test]
fn noun_phrase_recursion_can_branch_through_multiple_boxed_levels() {
    let phrase = dp("map")
        .the()
        .complement(pp(
            "of",
            dp("room")
                .the()
                .postmodifier(pp(
                    "inside",
                    dp("house")
                        .the()
                        .postmodifier(pp("near", dp("river").the())),
                )),
        ));

    assert_eq!(phrase.render(), "the map of the room inside the house near the river");
}

#[test]
fn agreement_tracks_default_singular_plural_and_counts() {
    assert_eq!(dp("child").agreement(), (Person::Third, Number::Singular));
    assert_eq!(dp("child").plural().agreement(), (Person::Third, Number::Plural));
    assert_eq!(dp("child").count(1).agreement(), (Person::Third, Number::Singular));
    assert_eq!(dp("child").count(2).agreement(), (Person::Third, Number::Plural));
    assert_eq!(dp("sheep").plural().agreement(), (Person::Third, Number::Plural));
}

#[test]
fn gap_dps_can_carry_full_semantics() {
    let gap = DeterminerPhrase::gap_with_semantics(
        DpSemantics::new(Person::Third, Number::Singular)
            .with_gender(Gender::Feminine)
            .with_animacy(Animacy::Animate),
    );

    assert_eq!(gap.render(), "");
    assert_eq!(gap.agreement(), (Person::Third, Number::Singular));
    assert_eq!(gap.reflexive_form(), "herself");
}

#[test]
fn simple_present_regular_verbs_agree_with_subject() {
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .affirmative()
            .subject(Person::First, Number::Singular)
            .render(),
        "eat"
    );
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Second, Number::Singular)
            .render(),
        "eat"
    );
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "eats"
    );
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .affirmative()
            .subject(Person::First, Number::Plural)
            .render(),
        "eat"
    );
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Second, Number::Plural)
            .render(),
        "eat"
    );
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "eat"
    );
}

#[test]
fn simple_past_regular_and_irregular_verbs_render() {
    assert_eq!(
        vp("walk")
            .past()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "walked"
    );
    assert_eq!(
        vp("eat")
            .past()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "ate"
    );
    assert_eq!(
        vp("go")
            .past()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "went"
    );
}

#[test]
fn simple_negative_regular_verbs_use_do_support() {
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .negative()
            .subject(Person::First, Number::Singular)
            .render(),
        "do not eat"
    );
    assert_eq!(
        vp("eat")
            .present()
            .simple()
            .negative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "does not eat"
    );
    assert_eq!(
        vp("eat")
            .past()
            .simple()
            .negative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "did not eat"
    );
}

#[test]
fn simple_be_uses_the_canonical_finite_paradigm() {
    assert_eq!(
        vp("be")
            .present()
            .simple()
            .affirmative()
            .subject(Person::First, Number::Singular)
            .render(),
        "am"
    );
    assert_eq!(
        vp("be")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Second, Number::Singular)
            .render(),
        "are"
    );
    assert_eq!(
        vp("be")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "is"
    );
    assert_eq!(
        vp("be")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "are"
    );
    assert_eq!(
        vp("be")
            .past()
            .simple()
            .affirmative()
            .subject(Person::First, Number::Singular)
            .render(),
        "was"
    );
    assert_eq!(
        vp("be")
            .past()
            .simple()
            .affirmative()
            .subject(Person::Second, Number::Singular)
            .render(),
        "were"
    );
    assert_eq!(
        vp("be")
            .past()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "were"
    );
}

#[test]
fn simple_be_negative_avoids_do_support() {
    assert_eq!(
        vp("be")
            .present()
            .simple()
            .negative()
            .subject(Person::First, Number::Singular)
            .render(),
        "am not"
    );
    assert_eq!(
        vp("be")
            .present()
            .simple()
            .negative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "is not"
    );
    assert_eq!(
        vp("be")
            .past()
            .simple()
            .negative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "were not"
    );
}

#[test]
fn present_perfect_varies_with_subject_agreement() {
    assert_eq!(
        vp("eat")
            .present()
            .perfect()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "has eaten"
    );
    assert_eq!(
        vp("eat")
            .present()
            .perfect()
            .affirmative()
            .subject(Person::First, Number::Plural)
            .render(),
        "have eaten"
    );
    assert_eq!(
        vp("eat")
            .present()
            .perfect()
            .negative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "has not eaten"
    );
}

#[test]
fn past_perfect_is_invariant_across_subjects() {
    assert_eq!(
        vp("eat")
            .past()
            .perfect()
            .affirmative()
            .subject(Person::First, Number::Singular)
            .render(),
        "had eaten"
    );
    assert_eq!(
        vp("eat")
            .past()
            .perfect()
            .negative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "had not eaten"
    );
}

#[test]
fn progressive_varies_with_be_agreement() {
    assert_eq!(
        vp("eat")
            .present()
            .progressive()
            .affirmative()
            .subject(Person::First, Number::Singular)
            .render(),
        "am eating"
    );
    assert_eq!(
        vp("eat")
            .present()
            .progressive()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "is eating"
    );
    assert_eq!(
        vp("eat")
            .present()
            .progressive()
            .negative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "are not eating"
    );
    assert_eq!(
        vp("eat")
            .past()
            .progressive()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "were eating"
    );
}

#[test]
fn perfect_progressive_renders_expected_auxiliary_chain() {
    assert_eq!(
        vp("eat")
            .present()
            .perfect_progressive()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "has been eating"
    );
    assert_eq!(
        vp("eat")
            .past()
            .perfect_progressive()
            .negative()
            .subject(Person::First, Number::Plural)
            .render(),
        "had not been eating"
    );
}

#[test]
fn modal_simple_forms_render_for_each_modal() {
    assert_eq!(vp("eat").modal(Modal::Will).simple().affirmative().render(), "will eat");
    assert_eq!(vp("eat").modal(Modal::Would).simple().affirmative().render(), "would eat");
    assert_eq!(vp("eat").modal(Modal::Can).simple().affirmative().render(), "can eat");
    assert_eq!(vp("eat").modal(Modal::Could).simple().affirmative().render(), "could eat");
    assert_eq!(vp("eat").modal(Modal::Shall).simple().affirmative().render(), "shall eat");
    assert_eq!(vp("eat").modal(Modal::Should).simple().affirmative().render(), "should eat");
    assert_eq!(vp("eat").modal(Modal::May).simple().affirmative().render(), "may eat");
    assert_eq!(vp("eat").modal(Modal::Might).simple().affirmative().render(), "might eat");
    assert_eq!(vp("eat").modal(Modal::Must).simple().affirmative().render(), "must eat");
}

#[test]
fn modal_negative_forms_insert_not_after_the_modal() {
    assert_eq!(vp("eat").modal(Modal::Will).simple().negative().render(), "will not eat");
    assert_eq!(vp("eat").modal(Modal::Would).simple().negative().render(), "would not eat");
    assert_eq!(vp("eat").modal(Modal::Can).simple().negative().render(), "can not eat");
    assert_eq!(vp("eat").modal(Modal::Should).simple().negative().render(), "should not eat");
    assert_eq!(vp("eat").modal(Modal::Must).simple().negative().render(), "must not eat");
}

#[test]
fn modal_perfect_forms_render_cleanly() {
    assert_eq!(
        vp("eat")
            .modal(Modal::Will)
            .perfect()
            .affirmative()
            .render(),
        "will have eaten"
    );
    assert_eq!(
        vp("eat")
            .modal(Modal::Would)
            .perfect()
            .negative()
            .render(),
        "would not have eaten"
    );
    assert_eq!(
        vp("go")
            .modal(Modal::Should)
            .perfect()
            .affirmative()
            .render(),
        "should have gone"
    );
}

#[test]
fn modal_progressive_forms_render_cleanly() {
    assert_eq!(
        vp("eat")
            .modal(Modal::Will)
            .progressive()
            .affirmative()
            .render(),
        "will be eating"
    );
    assert_eq!(
        vp("eat")
            .modal(Modal::Would)
            .progressive()
            .negative()
            .render(),
        "would not be eating"
    );
    assert_eq!(
        vp("eat")
            .modal(Modal::Must)
            .progressive()
            .affirmative()
            .render(),
        "must be eating"
    );
}

#[test]
fn modal_perfect_progressive_forms_render_cleanly() {
    assert_eq!(
        vp("eat")
            .modal(Modal::Will)
            .perfect_progressive()
            .affirmative()
            .render(),
        "will have been eating"
    );
    assert_eq!(
        vp("eat")
            .modal(Modal::Would)
            .perfect_progressive()
            .negative()
            .render(),
        "would not have been eating"
    );
}

#[test]
fn particles_survive_simple_perfect_progressive_and_modal_forms() {
    assert_eq!(
        vp("give")
            .particle("up")
            .present()
            .simple()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "gives up"
    );
    assert_eq!(
        vp("give")
            .particle("up")
            .present()
            .perfect()
            .negative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "has not given up"
    );
    assert_eq!(
        vp("look")
            .particle("up")
            .past()
            .progressive()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "were looking up"
    );
    assert_eq!(
        vp("give")
            .particle("up")
            .modal(Modal::Would)
            .perfect_progressive()
            .negative()
            .render(),
        "would not have been giving up"
    );
}

#[test]
fn clause_renders_subject_predicate_and_object() {
    assert_eq!(
        Clause::new(
            dp("child").the().plural(),
            vp("steal").past().simple().affirmative(),
        )
        .object(dp("potato").count(7))
        .render(),
        "the children stole 7 potatoes"
    );
    assert_eq!(
        Clause::new(
            dp("dog").the(),
            vp("bark").present().simple().affirmative(),
        )
        .render(),
        "the dog barks"
    );
}

#[test]
fn clause_supplies_subject_agreement_automatically() {
    assert_eq!(
        Clause::new(
            dp("child").plural(),
            vp("eat").present().simple().affirmative(),
        )
        .render(),
        "children eat"
    );
    assert_eq!(
        Clause::new(
            dp("child").the(),
            vp("eat").present().perfect().negative(),
        )
        .render(),
        "the child has not eaten"
    );
    assert_eq!(
        Clause::new(
            dp("child").the().plural(),
            vp("be").past().simple().negative(),
        )
        .render(),
        "the children were not"
    );
}

#[test]
fn clause_works_with_modified_subjects_and_objects() {
    let clause = Clause::new(
        dp("child")
            .the()
            .plural()
            .modifier(AdjPhrase::new("small").comparative()),
        vp("steal").past().simple().affirmative(),
    )
    .object(
        dp("potato")
            .count(7)
            .modifier(ap("red"))
            .postmodifier(pp("from", dp("cart").the())),
    );

    assert_eq!(clause.render(), "the smaller children stole 7 red potatoes from the cart");
}

#[test]
fn sentence_adds_capitalization_and_terminal_marks() {
    let clause = Clause::new(
        dp("child").the().plural(),
        vp("arrive").past().simple().affirmative(),
    );

    assert_eq!(clause.clone().sentence().render(), "the children arrived");
    assert_eq!(clause.clone().sentence().capitalize().render(), "The children arrived");
    assert_eq!(clause.clone().sentence().period().render(), "the children arrived.");
    assert_eq!(
        clause.clone().sentence().capitalize().period().render(),
        "The children arrived."
    );
    assert_eq!(
        clause.clone().sentence().capitalize().question_mark().render(),
        "The children arrived?"
    );
    assert_eq!(
        clause.sentence().capitalize().exclamation_mark().render(),
        "The children arrived!"
    );
}

#[test]
fn clause_and_sentence_handle_recursive_phrase_material() {
    let subject = dp("photo")
        .the()
        .complement(pp(
            "of",
            dp("child")
                .the()
                .postmodifier(pp("with", dp("toy").the())),
        ));

    let clause = Clause::new(
        subject,
        vp("hang").past().simple().affirmative(),
    )
    .prepositional(
        "on",
        dp("wall")
            .the()
            .postmodifier(pp("inside", dp("hall").the())),
    );

    assert_eq!(
        clause.render(),
        "the photo of the child with the toy hung on the wall inside the hall"
    );
    assert_eq!(
        clause.sentence().capitalize().period().render(),
        "The photo of the child with the toy hung on the wall inside the hall."
    );
}

#[test]
fn clause_supports_structured_prepositional_phrases() {
    let clause = Clause::new(
        dp("child").the().plural(),
        vp("wait").past().simple().affirmative(),
    )
    .prepositional(
        "near",
        dp("station")
            .the()
            .postmodifier(pp("by", dp("river").the())),
    );

    assert_eq!(clause.render(), "the children waited near the station by the river");
}
