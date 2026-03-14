use english_phrase::*;

#[test]
fn noun_phrase_example_from_api_design() {
    let np = NounPhrase::new("child")
        .determiner(Determiner::the())
        .modifier(AdjPhrase::new("small").comparative())
        .complement("from the next building")
        .plural();

    assert_eq!(np.render(), "the smaller children from the next building");
}

#[test]
fn adjective_phrase_example_from_api_design() {
    let adj = AdjPhrase::new("bad3")
        .comparative()
        .intensifier("far")
        .complement("than yesterday");

    assert_eq!(adj.render(), "far worse than yesterday");
}

#[test]
fn counted_noun_phrases_place_the_number_before_modifiers() {
    let np = NounPhrase::new("child")
        .count(3)
        .modifier("running")
        .complement("from the park");

    assert_eq!(np.render(), "3 running children from the park");
}

#[test]
fn adjective_phrases_work_as_noun_modifiers() {
    let np = NounPhrase::new("day")
        .the()
        .modifier(AdjPhrase::new("bad3").superlative());

    assert_eq!(np.render(), "the worst day");
}

#[test]
fn noun_phrase_agreement_tracks_count() {
    assert_eq!(NounPhrase::new("child").count(1).agreement(), (Person::Third, Number::Singular));
    assert_eq!(NounPhrase::new("child").count(2).agreement(), (Person::Third, Number::Plural));
}

#[test]
fn noun_phrase_supports_recursive_boxed_complements() {
    let np = NounPhrase::new("photo")
        .the()
        .complement("of")
        .complement(
            NounPhrase::new("child")
                .the()
                .complement("with")
                .complement(NounPhrase::new("toy").the()),
        );

    assert_eq!(np.render(), "the photo of the child with the toy");
}

#[test]
fn adjective_phrase_supports_boxed_noun_phrase_complements() {
    let adj = AdjPhrase::new("full")
        .positive()
        .complement("of")
        .complement(NounPhrase::new("bean").plural());

    assert_eq!(adj.render(), "full of beans");
}
