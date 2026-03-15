use english_phrase::*;

fn ap(adjective: &str) -> AdjPhrase {
    AdjPhrase::new(adjective)
}

fn pp(preposition: &str, complement: DeterminerPhrase) -> PrepositionalPhrase {
    PrepositionalPhrase::new(preposition, complement)
}

fn tp(subject: DeterminerPhrase, verb: &str) -> TensePhrase {
    TensePhrase::new(subject, VerbPhrase::new(verb).past().simple().affirmative())
}

#[test]
fn noun_phrase_example_from_api_design() {
    let np = DeterminerPhrase::new("child")
        .determiner(Determiner::the())
        .modifier(AdjPhrase::new("small").comparative())
        .postmodifier(pp(
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
        .complement(pp("than", DeterminerPhrase::proper_name("yesterday")));

    assert_eq!(adj.render(), "far worse than yesterday");
}

#[test]
fn counted_noun_phrases_place_the_number_before_modifiers() {
    let np = DeterminerPhrase::new("child")
        .count(3)
        .modifier(ap("running"))
        .postmodifier(pp("from", DeterminerPhrase::new("park").the()));

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
        .postmodifier(pp(
            "of",
            DeterminerPhrase::new("child")
                .the()
                .postmodifier(pp("with", DeterminerPhrase::new("toy").the())),
        ));

    assert_eq!(np.render(), "the photo of the child with the toy");
}

#[test]
fn adjective_phrase_supports_boxed_noun_phrase_complements() {
    let adj = AdjPhrase::new("full")
        .positive()
        .complement(pp("of", DeterminerPhrase::new("bean").plural()));

    assert_eq!(adj.render(), "full of beans");
}

#[test]
fn prepositional_phrase_is_a_first_class_public_phrase() {
    let pp = pp("on", DeterminerPhrase::new("wall").the());

    assert_eq!(pp.render(), "on the wall");
}

#[test]
fn prepositional_phrase_supports_recursive_phrase_structure() {
    let pp = pp(
        "on",
        DeterminerPhrase::new("wall")
            .the()
            .postmodifier(pp("inside", DeterminerPhrase::new("hall").the())),
    );

    assert_eq!(pp.render(), "on the wall inside the hall");
}

#[test]
fn adverb_phrase_is_a_first_class_public_phrase() {
    let degree = AdverbPhrase::new("far").specifier(AdverbPhrase::new("very"));
    let advp = AdverbPhrase::new("independently")
        .complement(pp("of", DeterminerPhrase::new("help")));

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
fn adjective_phrases_accept_typed_non_finite_clause_complements() {
    let adj = AdjPhrase::new("ready").complement(NonFiniteClause::to_infinitive(
        VerbPhrase::new("leave").simple().affirmative(),
    ));

    assert_eq!(adj.render(), "ready to leave");
}

#[test]
fn nominal_postmodifiers_accept_typed_non_finite_clauses() {
    let dp = DeterminerPhrase::new("decision")
        .the()
        .postmodifier(NonFiniteClause::to_infinitive(
            VerbPhrase::new("leave").simple().affirmative(),
        ));

    assert_eq!(dp.render(), "the decision to leave");
}

#[test]
fn nominal_postmodifiers_support_typed_relative_clauses() {
    let dp = DeterminerPhrase::new("child")
        .the()
        .postmodifier(
            RelativeClause::who(
                tp(DeterminerPhrase::gap(), "wait")
                    .prepositional("near", DeterminerPhrase::new("door").the()),
            ),
        );

    assert_eq!(dp.render(), "the child who waited near the door");
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
fn dp_semantics_cover_pronouns_proper_names_and_nominals() {
    assert_eq!(
        DeterminerPhrase::pronoun(Pronoun::he()).semantics(),
        &DpSemantics::new(Person::Third, Number::Singular)
            .with_gender(Gender::Masculine)
            .with_animacy(Animacy::Animate)
    );
    assert_eq!(
        DeterminerPhrase::proper_name("Alice")
            .feminine()
            .semantics(),
        &DpSemantics::new(Person::Third, Number::Singular)
            .with_gender(Gender::Feminine)
            .with_animacy(Animacy::Animate)
    );
    assert_eq!(
        DeterminerPhrase::new("rock").inanimate().semantics(),
        &DpSemantics::new(Person::Third, Number::Singular).with_animacy(Animacy::Inanimate)
    );
    assert_eq!(
        DeterminerPhrase::new("child").count(2).animate().semantics(),
        &DpSemantics::new(Person::Third, Number::Plural).with_animacy(Animacy::Animate)
    );
}

#[test]
fn dp_semantics_drive_reflexive_forms() {
    assert_eq!(
        DeterminerPhrase::pronoun(Pronoun::he()).reflexive_form(),
        "himself"
    );
    assert_eq!(
        DeterminerPhrase::pronoun(Pronoun::she()).reflexive_form(),
        "herself"
    );
    assert_eq!(
        DeterminerPhrase::pronoun(Pronoun::they()).reflexive_form(),
        "themselves"
    );
    assert_eq!(
        DeterminerPhrase::proper_name("Alice")
            .feminine()
            .reflexive_form(),
        "herself"
    );
    assert_eq!(
        DeterminerPhrase::new("machine").inanimate().reflexive_form(),
        "itself"
    );
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
