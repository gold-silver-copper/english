use english_phrase::*;

#[test]
fn adjective_phrase_renders_complex_forms() {
    let rendered = AdjPhrase::new("bad3")
        .degree(Degree::Comparative)
        .intensifier("far")
        .complement("than yesterday")
        .render();

    assert_eq!(rendered, "far worse than yesterday");
}

#[test]
fn noun_phrase_renders_modifiers() {
    let rendered = NounPhrase::new("child")
        .plural()
        .determiner("the")
        .modifier("running")
        .render();

    assert_eq!(rendered, "the running children");
}

#[test]
fn noun_phrase_supports_counted_complements() {
    let rendered = NounPhrase::new("pair")
        .count(3)
        .complement("of jeans")
        .render();

    assert_eq!(rendered, "3 pairs of jeans");
}

#[test]
fn adjective_phrase_can_be_used_as_a_noun_modifier() {
    let adjective = AdjPhrase::new("bad3").degree(Degree::Superlative).render();

    let rendered = NounPhrase::new("day")
        .singular()
        .determiner("the")
        .modifier_phrase(AdjPhrase::new("bad3").degree(Degree::Superlative))
        .render();

    assert_eq!(adjective, "worst");
    assert_eq!(rendered, "the worst day");
}

#[test]
fn noun_phrase_and_verb_phrase_compose_cleanly() {
    let subject = NounPhrase::new("child").plural().modifier("running");
    let object = NounPhrase::new("potato").count(7);

    let sentence = format!(
        "The {} {} {}.",
        subject.render(),
        VerbPhrase::new("steal")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Affirmative)
            .subject_noun_phrase(&subject)
            .render(),
        object.render()
    );

    assert_eq!(sentence, "The running children stole 7 potatoes.");
}
