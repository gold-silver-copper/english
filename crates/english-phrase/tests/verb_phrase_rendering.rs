use english_phrase::*;

fn main_verb(verb: impl Into<Verb>) -> VerbPhrase {
    VerbPhrase::new(verb)
}

#[test]
fn reexported_inflection_primitives_still_work() {
    assert_eq!(Verb::third_person("run"), "runs");
    assert_eq!(Verb::past("walk"), "walked");
    assert_eq!(Verb::present_participle("swim"), "swimming");
    assert_eq!(Verb::past_participle("eat"), "eaten");
    assert_eq!(Verb::infinitive("go"), "go");
}

#[test]
fn simple_present_affirmative_renders_with_subject_agreement() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "eats"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render(),
        "eat"
    );
}

#[test]
fn simple_past_affirmative_renders_regular_and_irregular_verbs() {
    assert_eq!(
        main_verb("walk")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "walked"
    );
    assert_eq!(
        main_verb("go")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "went"
    );
}

#[test]
fn simple_negative_uses_do_support_for_regular_verbs() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "does not eat"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render(),
        "did not eat"
    );
}

#[test]
fn simple_negative_does_not_use_do_support_for_be() {
    assert_eq!(
        main_verb("be")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "is not"
    );
    assert_eq!(
        main_verb("be")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render(),
        "were not"
    );
}

#[test]
fn modal_simple_renders_positive_and_negative_forms() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Will(ModalTense::Present))
            .render(),
        "will eat"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Will(ModalTense::Preterite))
            .render(),
        "would eat"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Can(ModalTense::Preterite))
            .render(),
        "could eat"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Can(ModalTense::Present))
            .render(),
        "can eat"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Shall(ModalTense::Preterite))
            .render(),
        "should eat"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .modal(Modal::Will(ModalTense::Present))
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "will not eat"
    );
}

#[test]
fn present_perfect_agrees_with_subject() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "has eaten"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render(),
        "have eaten"
    );
}

#[test]
fn perfect_negative_renders_after_have() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "has not eaten"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render(),
        "had not eaten"
    );
}

#[test]
fn modal_perfect_renders_future_and_conditional_forms() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Will(ModalTense::Present))
            .render(),
        "will have eaten"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .modal(Modal::Will(ModalTense::Preterite))
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "would not have eaten"
    );
}

#[test]
fn progressive_renders_present_and_past_be_forms() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "is eating"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render(),
        "were eating"
    );
}

#[test]
fn progressive_negative_renders_after_be() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "is not eating"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Progressive)
            .modal(Modal::Will(ModalTense::Present))
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "will not be eating"
    );
}

#[test]
fn perfect_progressive_renders_composed_auxiliaries() {
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::PerfectProgressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "has been eating"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::PerfectProgressive)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render(),
        "had not been eating"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::PerfectProgressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Will(ModalTense::Present))
            .render(),
        "will have been eating"
    );
}

#[test]
fn phrasal_verbs_survive_simple_and_complex_phrases() {
    let give_up = Verb::new("give").with_particle("up");
    let look_up = Verb::new("look").with_particle("up");

    assert_eq!(
        main_verb(give_up.clone())
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "gives up"
    );
    assert_eq!(
        main_verb(give_up)
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "has not given up"
    );
    assert_eq!(
        main_verb(look_up)
            .tense(BaseTense::Past)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Plural)
            .render(),
        "were looking up"
    );
}

#[test]
fn irregular_auxiliary_edge_cases_render_cleanly() {
    assert_eq!(
        main_verb("be")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render(),
        "has been"
    );
    assert_eq!(
        main_verb("have")
            .tense(BaseTense::Present)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render(),
        "are having"
    );
    assert_eq!(
        main_verb("go")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .modal(Modal::Shall(ModalTense::Preterite))
            .render(),
        "should have gone"
    );
}
