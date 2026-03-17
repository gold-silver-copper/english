use english_phrase::*;
use english_phrase::syntax::TpForm;

fn assert_sentence<Form: TpForm>(clause: TensePhrase<Form>, expected: &str) {
    assert_eq!(
        clause.realize_with(RealizationOptions::sentence()),
        expected
    );
}

fn assert_with_options<Form: TpForm>(
    clause: TensePhrase<Form>,
    options: RealizationOptions,
    expected: &str,
) {
    assert_eq!(clause.realize_with(options), expected);
}

fn long_editor_subject() -> DeterminerPhrase {
    dp(np("editor")
        .modifier(adjp("patient").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("office").modifier(adjp("northern"))).the(),
        )))
    .the()
}

fn long_editors_subject() -> DeterminerPhrase {
    dp(np("editor")
        .plural()
        .modifier(adjp("patient").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("office").modifier(adjp("northern"))).the(),
        )))
    .these()
}

fn careful_guide_object() -> DeterminerPhrase {
    dp(np("guide")
        .modifier(adjp("careful").modifier(advp("unusually")))
        .complement(pp(
            "from",
            dp(np("village").modifier(adjp("coastal"))).the(),
        )))
    .the()
}

#[test]
fn present_tense_agreement_surfaces_on_a_long_third_person_singular_clause() {
    let clause = tp(vp("admire")
        .complement(careful_guide_object())
        .adjunct(pp(
            "after",
            dp(np("rehearsal").modifier(adjp("long"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .present()
    .subject(long_editor_subject());

    assert_sentence(
        clause,
        "The remarkably patient editor from the northern office admires the unusually careful guide from the coastal village after the long rehearsal near the harbor.",
    );
}

#[test]
fn present_tense_agreement_surfaces_on_a_long_third_person_plural_clause() {
    let clause = tp(vp("admire")
        .complement(careful_guide_object())
        .adjunct(pp(
            "after",
            dp(np("rehearsal").modifier(adjp("long"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .present()
    .subject(long_editors_subject());

    assert_sentence(
        clause,
        "These remarkably patient editors from the northern office admire the unusually careful guide from the coastal village after the long rehearsal near the harbor.",
    );
}

#[test]
fn present_negative_uses_do_support_inside_a_long_sentence() {
    let clause = tp(vp("admire")
        .complement(careful_guide_object())
        .adjunct(pp(
            "after",
            dp(np("rehearsal").modifier(adjp("long"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .present()
    .negative()
    .subject(long_editor_subject());

    assert_sentence(
        clause,
        "The remarkably patient editor from the northern office does not admire the unusually careful guide from the coastal village after the long rehearsal near the harbor.",
    );
}

#[test]
fn past_negative_uses_did_support_inside_a_long_sentence() {
    let clause = tp(vp("admire")
        .complement(careful_guide_object())
        .adjunct(pp(
            "after",
            dp(np("rehearsal").modifier(adjp("long"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .past()
    .negative()
    .subject(long_editors_subject());

    assert_sentence(
        clause,
        "These remarkably patient editors from the northern office did not admire the unusually careful guide from the coastal village after the long rehearsal near the harbor.",
    );
}

#[test]
fn present_be_surfaces_cleanly_inside_a_long_sentence() {
    let clause = tp(vp("be").complement(
        adjp("ready").complement(
            tp(vp("repair")
                .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        ),
    ))
    .present()
    .subject(
        dp(np("mechanic")
            .modifier(adjp("careful").modifier(advp("unusually")))
            .complement(pp(
                "from",
                dp(np("workshop").modifier(adjp("northern"))).the(),
            )))
        .the(),
    );

    assert_sentence(
        clause,
        "The unusually careful mechanic from the northern workshop is ready to repair the damaged engine before the final inspection.",
    );
}

#[test]
fn past_negative_be_surfaces_cleanly_inside_a_long_sentence() {
    let clause = tp(vp("be").complement(
        adjp("ready").complement(
            tp(vp("repair")
                .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        ),
    ))
    .past()
    .negative()
    .subject(
        dp(np("mechanic")
            .plural()
            .modifier(adjp("careful").modifier(advp("unusually")))
            .complement(pp(
                "from",
                dp(np("workshop").modifier(adjp("northern"))).the(),
            )))
        .those(),
    );

    assert_sentence(
        clause,
        "Those unusually careful mechanics from the northern workshop were not ready to repair the damaged engine before the final inspection.",
    );
}

#[test]
fn gerund_participle_clause_stays_grammatical_inside_a_long_sentence() {
    let clause = tp(vp("discuss")
        .complement(
            tp(vp("repair")
                .complement(dp(np("bridge").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("winter"))).the(),
                )))
            .gerund_participle(),
        )
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(dp(np("council").modifier(adjp("local"))).the());

    assert_sentence(
        clause,
        "The local council discussed repairing the damaged bridge before the winter inspection after the storm.",
    );
}

#[test]
fn negative_gerund_participle_clause_stays_grammatical_inside_a_long_sentence() {
    let clause = tp(vp("discuss")
        .complement(
            tp(vp("repair")
                .complement(dp(np("bridge").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("winter"))).the(),
                )))
            .gerund_participle()
            .negative(),
        )
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(dp(np("council").modifier(adjp("local"))).the());

    assert_sentence(
        clause,
        "The local council discussed not repairing the damaged bridge before the winter inspection after the storm.",
    );
}

#[test]
fn sentence_options_capitalize_and_punctuate_a_long_clause() {
    let clause = tp(vp("expect")
        .complement(
            dp(np("mechanic")
                .modifier(adjp("careful").modifier(advp("unusually")))
                .complement(pp(
                    "from",
                    dp(np("workshop").modifier(adjp("northern"))).the(),
                )))
            .the(),
        )
        .complement(
            tp(vp("repair")
                .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        )
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(dp(np("council").modifier(adjp("local"))).the());

    assert_with_options(
        clause,
        RealizationOptions::sentence(),
        "The local council expected the unusually careful mechanic from the northern workshop to repair the damaged engine before the final inspection after the storm.",
    );
}

#[test]
fn question_mark_option_changes_only_the_terminal_on_a_long_clause() {
    let clause = tp(vp("expect")
        .complement(
            dp(np("mechanic")
                .modifier(adjp("careful").modifier(advp("unusually")))
                .complement(pp(
                    "from",
                    dp(np("workshop").modifier(adjp("northern"))).the(),
                )))
            .the(),
        )
        .complement(
            tp(vp("repair")
                .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        )
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(dp(np("council").modifier(adjp("local"))).the());

    assert_with_options(
        clause,
        RealizationOptions::default().capitalize().question_mark(),
        "The local council expected the unusually careful mechanic from the northern workshop to repair the damaged engine before the final inspection after the storm?",
    );
}

#[test]
fn exclamation_mark_option_changes_only_the_terminal_on_a_long_clause() {
    let clause = tp(vp("expect")
        .complement(
            dp(np("mechanic")
                .modifier(adjp("careful").modifier(advp("unusually")))
                .complement(pp(
                    "from",
                    dp(np("workshop").modifier(adjp("northern"))).the(),
                )))
            .the(),
        )
        .complement(
            tp(vp("repair")
                .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        )
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(dp(np("council").modifier(adjp("local"))).the());

    assert_with_options(
        clause,
        RealizationOptions::default()
            .capitalize()
            .exclamation_mark(),
        "The local council expected the unusually careful mechanic from the northern workshop to repair the damaged engine before the final inspection after the storm!",
    );
}

#[test]
fn lowercase_without_terminal_leaves_a_long_clause_as_plain_text() {
    let clause = tp(vp("expect")
        .complement(
            dp(np("mechanic")
                .modifier(adjp("careful").modifier(advp("unusually")))
                .complement(pp(
                    "from",
                    dp(np("workshop").modifier(adjp("northern"))).the(),
                )))
            .the(),
        )
        .complement(
            tp(vp("repair")
                .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        )
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(dp(np("council").modifier(adjp("local"))).the());

    assert_with_options(
        clause,
        RealizationOptions::default().lowercase().without_terminal(),
        "the local council expected the unusually careful mechanic from the northern workshop to repair the damaged engine before the final inspection after the storm",
    );
}
