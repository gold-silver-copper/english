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
    let subject = NounPhrase::new("child").the().plural();

    let vp = VerbPhrase::new("eat")
        .present()
        .perfect()
        .negative()
        .agree_with(&subject);

    assert_eq!(vp.render(), "have not eaten");
}

#[test]
fn clause_and_sentence_example_from_api_design() {
    let clause = Clause::new(
        NounPhrase::new("child").the().plural(),
        VerbPhrase::new("steal").past().simple().affirmative(),
    )
    .object(NounPhrase::new("potato").count(7));

    assert_eq!(clause.render(), "the children stole 7 potatoes");
    assert_eq!(
        clause.clone().sentence().capitalize().period().render(),
        "The children stole 7 potatoes."
    );
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
fn clause_supplies_subject_agreement_to_the_predicate() {
    let clause = Clause::new(
        NounPhrase::new("child").plural(),
        VerbPhrase::new("eat").present().simple().affirmative(),
    );

    assert_eq!(clause.render(), "children eat");
}

#[test]
fn sentence_supports_multiple_terminal_styles() {
    let clause = Clause::new(
        NounPhrase::new("child").the().plural(),
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
