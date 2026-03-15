use english_phrase::*;

#[test]
fn wrappers_are_reexported() {
    assert_eq!(Verb::new("run").third_person(), "runs");
    assert_eq!(Verb::new("walk").past(), "walked");
    assert_eq!(Verb::new("swim").present_participle(), "swimming");
    assert_eq!(Verb::new("eat").past_participle(), "eaten");
    assert_eq!(Verb::new("go").infinitive(), "go");
}

#[test]
fn verb_phrase_example_from_api_design() {
    let subject = DeterminerPhrase::new("child").the().plural();

    let vp = VerbPhrase::new("eat")
        .present()
        .perfect()
        .negative()
        .agree_with(&subject);

    assert_eq!(vp.render(), "have not eaten");
}

#[test]
fn verb_phrase_renders_typed_complements_and_adjuncts() {
    let vp = VerbPhrase::new("give")
        .past()
        .simple()
        .affirmative()
        .subject(Person::Third, Number::Singular)
        .direct_object(DeterminerPhrase::new("apple").the())
        .prepositional_adjunct(PrepositionalPhrase::new(
            "to",
            DeterminerPhrase::new("child").the(),
        ));

    assert_eq!(vp.render(), "gave the apple to the child");
}

#[test]
fn verb_phrase_supports_adjective_and_clausal_complements() {
    let copular = VerbPhrase::new("be")
        .present()
        .simple()
        .affirmative()
        .subject(Person::Third, Number::Singular)
        .predicative_complement(AdjPhrase::new("ready"))
        .non_finite_complement(NonFiniteClause::to_infinitive(
            VerbPhrase::new("leave").simple().affirmative(),
        ));

    let clausal = VerbPhrase::new("say")
        .past()
        .simple()
        .affirmative()
        .subject(Person::Third, Number::Singular)
        .clausal_complement(ComplementizerPhrase::that(TensePhrase::new(
            DeterminerPhrase::new("child").the().plural(),
            VerbPhrase::new("arrive").past().simple().affirmative(),
        )));

    assert_eq!(copular.render(), "is ready to leave");
    assert_eq!(clausal.render(), "said that the children arrived");
}

#[test]
fn clause_and_sentence_example_from_api_design() {
    let clause = Clause::new(
        DeterminerPhrase::new("child").the().plural(),
        VerbPhrase::new("steal").past().simple().affirmative(),
    )
    .object(DeterminerPhrase::new("potato").count(7));

    assert_eq!(clause.render(), "the children stole 7 potatoes");
    assert_eq!(
        clause.clone().sentence().capitalize().period().render(),
        "The children stole 7 potatoes."
    );
}

#[test]
fn tense_phrase_is_a_first_class_public_projection() {
    let tp = TensePhrase::new(
        DeterminerPhrase::new("child").the().plural(),
        VerbPhrase::new("steal").past().simple().affirmative(),
    )
    .object(DeterminerPhrase::new("potato").count(7))
    .prepositional("from", DeterminerPhrase::new("market").the());

    assert_eq!(tp.render(), "the children stole 7 potatoes from the market");
    assert_eq!(
        tp.clone().sentence().capitalize().period().render(),
        "The children stole 7 potatoes from the market."
    );
    assert_eq!(
        Clause::from_tense_phrase(tp).render(),
        "the children stole 7 potatoes from the market"
    );
}

#[test]
fn complementizer_phrase_is_a_first_class_public_projection() {
    let cp = ComplementizerPhrase::that(TensePhrase::new(
        DeterminerPhrase::new("child").the().plural(),
        VerbPhrase::new("arrive").past().simple().affirmative(),
    ));

    assert_eq!(cp.render(), "that the children arrived");
}

#[test]
fn non_finite_clause_is_a_first_class_public_projection() {
    let clause = NonFiniteClause::to_infinitive(
        VerbPhrase::new("eat").perfect().negative(),
    )
    .object(DeterminerPhrase::new("apple").the())
    .prepositional("in", DeterminerPhrase::new("garden").the());

    assert_eq!(clause.render(), "not to have eaten the apple in the garden");
}

#[test]
fn simple_negative_uses_do_support() {
    assert_eq!(
        VerbPhrase::new("eat")
            .present()
            .simple()
            .negative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "does not eat"
    );
    assert_eq!(
        VerbPhrase::new("eat")
            .past()
            .simple()
            .negative()
            .subject(Person::First, Number::Plural)
            .render(),
        "did not eat"
    );
}

#[test]
fn be_uses_the_canonical_finite_paradigm() {
    assert_eq!(
        VerbPhrase::new("be")
            .present()
            .simple()
            .negative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "is not"
    );
    assert_eq!(
        VerbPhrase::new("be")
            .past()
            .simple()
            .negative()
            .subject(Person::First, Number::Plural)
            .render(),
        "were not"
    );
}

#[test]
fn modal_and_particle_rendering_still_work() {
    assert_eq!(
        VerbPhrase::new("give")
            .particle("up")
            .modal(Modal::Would)
            .perfect()
            .negative()
            .render(),
        "would not have given up"
    );
    assert_eq!(
        VerbPhrase::new("look")
            .particle("up")
            .past()
            .progressive()
            .affirmative()
            .subject(Person::Third, Number::Plural)
            .render(),
        "were looking up"
    );
}

#[test]
fn present_perfect_agrees_with_subject_number() {
    assert_eq!(
        VerbPhrase::new("eat")
            .present()
            .perfect()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "has eaten"
    );
    assert_eq!(
        VerbPhrase::new("eat")
            .present()
            .perfect()
            .affirmative()
            .subject(Person::First, Number::Plural)
            .render(),
        "have eaten"
    );
}

#[test]
fn progressive_and_perfect_progressive_render_cleanly() {
    assert_eq!(
        VerbPhrase::new("eat")
            .present()
            .progressive()
            .affirmative()
            .subject(Person::Third, Number::Singular)
            .render(),
        "is eating"
    );
    assert_eq!(
        VerbPhrase::new("eat")
            .past()
            .perfect_progressive()
            .negative()
            .subject(Person::First, Number::Plural)
            .render(),
        "had not been eating"
    );
}

#[test]
fn modal_perfect_and_modal_progressive_render_cleanly() {
    assert_eq!(
        VerbPhrase::new("eat")
            .modal(Modal::Will)
            .perfect()
            .affirmative()
            .render(),
        "will have eaten"
    );
    assert_eq!(
        VerbPhrase::new("eat")
            .modal(Modal::Would)
            .progressive()
            .negative()
            .render(),
        "would not be eating"
    );
}

#[test]
fn passive_voice_renders_cleanly_across_aspects() {
    assert_eq!(
        VerbPhrase::new("praise")
            .past()
            .simple()
            .passive()
            .subject(Person::Third, Number::Singular)
            .render(),
        "was praised"
    );
    assert_eq!(
        VerbPhrase::new("praise")
            .present()
            .perfect()
            .passive()
            .subject(Person::Third, Number::Singular)
            .render(),
        "has been praised"
    );
    assert_eq!(
        VerbPhrase::new("praise")
            .modal(Modal::Would)
            .progressive()
            .negative()
            .passive()
            .render(),
        "would not be being praised"
    );
}

#[test]
fn tense_phrase_passive_promotes_object_and_demotes_subject() {
    let tp = TensePhrase::new(
        DeterminerPhrase::new("teacher").the(),
        VerbPhrase::new("praise").past().simple().affirmative(),
    )
    .object(DeterminerPhrase::new("child").the())
    .passive();

    assert_eq!(tp.render(), "the child was praised by the teacher");
}

#[test]
fn tense_phrase_causative_restructures_around_make() {
    let tp = TensePhrase::new(
        DeterminerPhrase::new("child").the(),
        VerbPhrase::new("eat").past().simple().affirmative(),
    )
    .object(DeterminerPhrase::new("apple").the())
    .prepositional("in", DeterminerPhrase::new("garden").the())
    .causative(DeterminerPhrase::new("teacher").the());

    assert_eq!(
        tp.render(),
        "the teacher made the child eat the apple in the garden"
    );
}

#[test]
fn clause_supplies_subject_agreement_to_the_predicate() {
    let clause = Clause::new(
        DeterminerPhrase::new("child").plural(),
        VerbPhrase::new("eat").present().simple().affirmative(),
    );

    assert_eq!(clause.render(), "children eat");
}

#[test]
fn sentence_supports_multiple_terminal_styles() {
    let clause = Clause::new(
        DeterminerPhrase::new("child").the().plural(),
        VerbPhrase::new("arrive").past().simple().affirmative(),
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
fn pronoun_and_proper_name_dps_supply_agreement_to_verbs() {
    assert_eq!(
        VerbPhrase::new("eat")
            .present()
            .simple()
            .affirmative()
            .agree_with(&DeterminerPhrase::pronoun(Pronoun::they()))
            .render(),
        "eat"
    );
    assert_eq!(
        VerbPhrase::new("eat")
            .present()
            .simple()
            .affirmative()
            .agree_with(&DeterminerPhrase::proper_name("Alice"))
            .render(),
        "eats"
    );
}
