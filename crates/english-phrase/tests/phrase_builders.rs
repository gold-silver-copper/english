use english_phrase::*;

fn ap(adjective: &str) -> AdjPhrase {
    AdjPhrase::new(adjective)
}

#[test]
fn noun_phrase_example_from_api_design() {
    let np = DeterminerPhrase::new("child")
        .determiner(Determiner::the())
        .modifier(AdjPhrase::new("small").comparative())
        .postmodifier(PrepositionalPhrase::new(
            "from",
            DeterminerPhrase::new("building")
                .the()
                .modifier(ap("next")),
        ))
        .plural();

    assert_eq!(np.render(), "the smaller children from the next building");
}

#[test]
fn adjective_phrase_example_from_api_design() {
    let adj = AdjPhrase::new("bad3")
        .comparative()
        .intensifier(AdverbPhrase::new("far"))
        .complement(PrepositionalPhrase::new(
            "than",
            DeterminerPhrase::proper_name("yesterday"),
        ));

    assert_eq!(adj.render(), "far worse than yesterday");
}

#[test]
fn counted_noun_phrases_place_the_number_before_modifiers() {
    let np = DeterminerPhrase::new("child")
        .count(3)
        .modifier(ap("running"))
        .postmodifier(PrepositionalPhrase::new(
            "from",
            DeterminerPhrase::new("park").the(),
        ));

    assert_eq!(np.render(), "3 running children from the park");
}

#[test]
fn adjective_phrases_work_as_noun_modifiers() {
    let np = DeterminerPhrase::new("day")
        .the()
        .modifier(AdjPhrase::new("bad3").superlative());

    assert_eq!(np.render(), "the worst day");
}

#[test]
fn noun_phrase_agreement_tracks_count() {
    assert_eq!(DeterminerPhrase::new("child").count(1).agreement(), (Person::Third, Number::Singular));
    assert_eq!(DeterminerPhrase::new("child").count(2).agreement(), (Person::Third, Number::Plural));
}

#[test]
fn noun_phrase_supports_recursive_boxed_complements() {
    let np = DeterminerPhrase::new("photo")
        .the()
        .postmodifier(PrepositionalPhrase::new(
            "of",
            DeterminerPhrase::new("child")
                .the()
                .postmodifier(PrepositionalPhrase::new(
                    "with",
                    DeterminerPhrase::new("toy").the(),
                )),
        ));

    assert_eq!(np.render(), "the photo of the child with the toy");
}

#[test]
fn adjective_phrase_supports_boxed_noun_phrase_complements() {
    let adj = AdjPhrase::new("full")
        .positive()
        .complement(PrepositionalPhrase::new(
            "of",
            DeterminerPhrase::new("bean").plural(),
        ));

    assert_eq!(adj.render(), "full of beans");
}

#[test]
fn prepositional_phrase_is_a_first_class_public_phrase() {
    let pp = PrepositionalPhrase::new("on", DeterminerPhrase::new("wall").the());

    assert_eq!(pp.render(), "on the wall");
}

#[test]
fn prepositional_phrase_supports_recursive_phrase_structure() {
    let pp = PrepositionalPhrase::new(
        "on",
        DeterminerPhrase::new("wall")
            .the()
            .postmodifier(
                PrepositionalPhrase::new("inside", DeterminerPhrase::new("hall").the()),
            ),
    );

    assert_eq!(pp.render(), "on the wall inside the hall");
}

#[test]
fn adverb_phrase_is_a_first_class_public_phrase() {
    let degree = AdverbPhrase::new("far").specifier(AdverbPhrase::new("very"));
    let advp = AdverbPhrase::new("independently")
        .complement(PrepositionalPhrase::new("of", DeterminerPhrase::new("help")));

    assert_eq!(degree.render(), "very far");
    assert_eq!(advp.render(), "independently of help");
}

#[test]
fn adjective_phrase_accepts_adverb_phrase_as_its_intensifier() {
    let adj = AdjPhrase::new("small")
        .comparative()
        .intensifier(AdverbPhrase::new("much"));

    assert_eq!(adj.render(), "much smaller");
}

#[test]
fn nominal_postmodifiers_support_typed_relative_clauses() {
    let dp = DeterminerPhrase::new("child")
        .the()
        .postmodifier(RelativeClause::who("waited outside"));

    assert_eq!(dp.render(), "the child who waited outside");
}

#[test]
fn pronoun_dps_render_as_full_determiner_phrases() {
    assert_eq!(DeterminerPhrase::pronoun(Pronoun::they()).render(), "they");
    assert_eq!(DeterminerPhrase::pronoun(Pronoun::he()).render(), "he");
}

#[test]
fn proper_name_dps_render_as_full_determiner_phrases() {
    assert_eq!(DeterminerPhrase::proper_name("Alice").render(), "Alice");
    assert_eq!(DeterminerPhrase::proper_name("James").render(), "James");
}

#[test]
fn possessors_render_in_spec_dp_position() {
    let johns_book = DeterminerPhrase::new("book")
        .possessor(DeterminerPhrase::proper_name("John"))
        .render();
    let their_house = DeterminerPhrase::new("house")
        .possessor(DeterminerPhrase::pronoun(Pronoun::they()))
        .render();
    let james_book = DeterminerPhrase::new("book")
        .possessor(DeterminerPhrase::proper_name("James"))
        .render();

    assert_eq!(johns_book, "John's book");
    assert_eq!(their_house, "their house");
    assert_eq!(james_book, "James' book");
}
