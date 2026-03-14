use english_phrase::*;

#[test]
fn adjective_phrase_requires_degree_and_renders_complex_forms() {
    assert_eq!(
        AdjPhrase::new("fast").render(),
        Err(AdjPhraseError::MissingDegree)
    );

    let rendered = AdjPhrase::new("bad3")
        .degree(Degree::Comparative)
        .intensifier("far")
        .complement("than yesterday")
        .render()
        .unwrap();

    assert_eq!(rendered, "far worse than yesterday");
}

#[test]
fn noun_phrase_requires_number_or_count_and_renders_modifiers() {
    assert_eq!(
        NounPhrase::new("child").render(),
        Err(NounPhraseError::MissingNumber)
    );

    let rendered = NounPhrase::new("child")
        .number(Number::Plural)
        .determiner("the")
        .modifier("running")
        .render()
        .unwrap();

    assert_eq!(rendered, "the running children");
}

#[test]
fn noun_phrase_count_mismatch_is_explicit() {
    assert_eq!(
        NounPhrase::new("child")
            .count(1)
            .number(Number::Plural)
            .render(),
        Err(NounPhraseError::CountNumberMismatch {
            count: 1,
            number: Number::Plural,
        })
    );
}

#[test]
fn noun_phrase_supports_counted_complements() {
    let rendered = NounPhrase::new("pair")
        .count(3)
        .complement("of jeans")
        .render()
        .unwrap();

    assert_eq!(rendered, "3 pairs of jeans");
}

#[test]
fn adjective_phrase_can_be_used_as_a_noun_modifier() {
    let adjective = AdjPhrase::new("bad3")
        .degree(Degree::Superlative)
        .render()
        .unwrap();

    let rendered = NounPhrase::new("day")
        .number(Number::Singular)
        .determiner("the")
        .modifier_phrase(AdjPhrase::new("bad3").degree(Degree::Superlative))
        .render()
        .unwrap();

    assert_eq!(adjective, "worst");
    assert_eq!(rendered, "the worst day");
}

#[test]
fn noun_phrase_and_verb_phrase_compose_cleanly() {
    let subject = NounPhrase::new("child")
        .number(Number::Plural)
        .modifier("running");
    let object = NounPhrase::new("potato").count(7);

    let sentence = format!(
        "The {} {} {}.",
        subject.render().unwrap(),
        VerbPhrase::new("steal")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject_noun_phrase(&subject)
            .render()
            .unwrap(),
        object.render().unwrap()
    );

    assert_eq!(sentence, "The running children stole 7 potatoes.");
}
