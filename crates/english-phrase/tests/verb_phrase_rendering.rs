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
            .render()
            .unwrap(),
        "eats"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "walked"
    );
    assert_eq!(
        main_verb("go")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "does not eat"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "is not"
    );
    assert_eq!(
        main_verb("be")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
        "were not"
    );
}

#[test]
fn modal_simple_renders_positive_and_negative_forms() {
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Will)
            .render()
            .unwrap(),
        "will eat"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Would)
            .render()
            .unwrap(),
        "would eat"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Could)
            .render()
            .unwrap(),
        "could eat"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Can)
            .render()
            .unwrap(),
        "can eat"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Should)
            .render()
            .unwrap(),
        "should eat"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Simple)
            .modal(Modal::Will)
            .polarity(Polarity::Negative)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "has eaten"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "has not eaten"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
        "had not eaten"
    );
}

#[test]
fn modal_perfect_renders_future_and_conditional_forms() {
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Will)
            .render()
            .unwrap(),
        "will have eaten"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Perfect)
            .modal(Modal::Would)
            .polarity(Polarity::Negative)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "is eating"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "is not eating"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::Progressive)
            .modal(Modal::Will)
            .polarity(Polarity::Negative)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "has been eating"
    );
    assert_eq!(
        main_verb("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::PerfectProgressive)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
        "had not been eating"
    );
    assert_eq!(
        main_verb("eat")
            .aspect(Aspect::PerfectProgressive)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Will)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "gives up"
    );
    assert_eq!(
        main_verb(give_up)
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render()
            .unwrap(),
        "has not given up"
    );
    assert_eq!(
        main_verb(look_up)
            .tense(BaseTense::Past)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Plural)
            .render()
            .unwrap(),
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
            .render()
            .unwrap(),
        "has been"
    );
    assert_eq!(
        main_verb("have")
            .tense(BaseTense::Present)
            .aspect(Aspect::Progressive)
            .polarity(Polarity::Affirmative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap(),
        "are having"
    );
    assert_eq!(
        main_verb("go")
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .modal(Modal::Should)
            .render()
            .unwrap(),
        "should have gone"
    );
}
