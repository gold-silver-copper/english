use english_phrase::*;

fn assert_sentence(clause: TensePhrase, expected: &str) {
    assert_eq!(
        clause.realize_with(RealizationOptions::sentence()).unwrap(),
        expected
    );
}

#[test]
fn careful_plan_to_repair_bridge_impressed_council() {
    let subject = dp(np("plan").modifier(adjp("careful")).complement(
        tp(vp("repair")
            .complement(dp(np("bridge").modifier(adjp("old"))).the())
            .adjunct(pp("before", dp(np("storm")).the())))
        .to_infinitive(),
    ))
    .the();

    let clause = tp(vp("impress").complement(dp(np("council").modifier(adjp("local"))).the()))
        .past()
        .subject(subject);

    assert_sentence(
        clause,
        "The careful plan to repair the old bridge before the storm impressed the local council.",
    );
}

#[test]
fn we_did_not_expect_editor_to_read_manuscript_on_train() {
    let subject = dp(Pronoun::We);

    let clause = tp(vp("expect")
        .complement(
            dp(np("editor")
                .modifier(adjp("patient").modifier(advp("remarkably")))
                .complement(pp("with", dp(np("lantern")).indefinite())))
            .the(),
        )
        .complement(
            tp(vp("read")
                .complement(
                    dp(np("manuscript").modifier(adjp("long").modifier(advp("very")))).the(),
                )
                .adjunct(pp("on", dp(np("train")).the())))
            .to_infinitive(),
        ))
    .past()
    .negative()
    .subject(subject);

    assert_sentence(
        clause,
        "We did not expect the remarkably patient editor with a lantern to read the very long manuscript on the train.",
    );
}

#[test]
fn alice_mailed_report_about_storm_to_office_near_harbor() {
    let subject = dp(name("Alice"));

    let clause = tp(vp("mail")
        .complement(
            dp(np("report")
                .modifier(adjp("detailed").modifier(advp("unusually")))
                .complement(pp(
                    "about",
                    dp(np("storm")
                        .complement(pp("over", dp(np("coast").modifier(adjp("northern"))).the())))
                    .the(),
                )))
            .the(),
        )
        .adjunct(pp(
            "to",
            dp(np("office")
                .modifier(adjp("quiet"))
                .complement(pp("near", dp(np("harbor")).the())))
            .the(),
        )))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "Alice mailed the unusually detailed report about the storm over the northern coast to the quiet office near the harbor.",
    );
}

#[test]
fn old_machine_under_stairs_was_not_ready_to_move() {
    let subject = dp(np("machine")
        .modifier(adjp("old"))
        .complement(pp("under", dp(np("stairs")).the())))
    .the();

    let clause =
        tp(vp("be").complement(adjp("ready").complement(
            tp(vp("move").adjunct(pp("into", dp(np("workshop")).the()))).to_infinitive(),
        )))
        .past()
        .negative()
        .subject(subject);

    assert_sentence(
        clause,
        "The old machine under the stairs was not ready to move into the workshop.",
    );
}

#[test]
fn very_nearly_impossible_puzzle_confused_children_in_library() {
    let subject = dp(np("puzzle")
        .modifier(adjp("impossible").modifier(advp("nearly").modifier(advp("very"))))
        .complement(pp("from", dp(np("museum")).the())))
    .the();

    let clause = tp(vp("confuse")
        .complement(
            dp(np("child")
                .plural()
                .modifier(adjp("patient").modifier(advp("extremely"))))
            .the(),
        )
        .adjunct(pp("in", dp(np("library")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The very nearly impossible puzzle from the museum confused the extremely patient children in the library.",
    );
}

#[test]
fn they_discussed_mapping_cave_with_old_guide_after_meal() {
    let subject = dp(Pronoun::They);

    let clause = tp(vp("discuss")
        .complement(
            tp(vp("map")
                .complement(
                    dp(np("cave")
                        .modifier(adjp("narrow"))
                        .complement(pp("under", dp(np("hill")).the())))
                    .the(),
                )
                .adjunct(pp("with", dp(np("guide").modifier(adjp("old"))).the())))
            .gerund_participle(),
        )
        .adjunct(pp("after", dp(np("meal")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "They discussed mapping the narrow cave under the hill with the old guide after the meal.",
    );
}

#[test]
fn ambitious_attempt_to_persuade_pilot_to_land_plane_alarmed_team() {
    let subject = dp(np("attempt").modifier(adjp("ambitious")).complement(
        tp(vp("persuade")
            .complement(dp(np("pilot").modifier(adjp("cautious"))).the())
            .complement(
                tp(vp("land")
                    .complement(dp(np("plane").modifier(adjp("damaged"))).the())
                    .adjunct(pp("near", dp(np("village")).the())))
                .to_infinitive(),
            ))
        .to_infinitive(),
    ))
    .the();

    let clause = tp(vp("alarm").complement(dp(np("team").modifier(adjp("rescue"))).the()))
        .past()
        .subject(subject);

    assert_sentence(
        clause,
        "The ambitious attempt to persuade the cautious pilot to land the damaged plane near the village alarmed the rescue team.",
    );
}
