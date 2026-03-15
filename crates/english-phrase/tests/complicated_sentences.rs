use english_phrase::*;

fn assert_sentence(sentence: Sentence, expected: &str) {
    assert_eq!(sentence.realize().unwrap(), expected);
}

#[test]
fn careful_plan_to_repair_bridge_impressed_council() {
    let subject = dp(np("plan").modifier(adjp("careful")).complement(
        vp("repair")
            .to_infinitive()
            .complement(dp(np("bridge").modifier(adjp("old"))).the())
            .adjunct(pp("before", dp(np("storm")).the())),
    ))
    .the();

    let predicate = vp("impress")
        .past()
        .complement(dp(np("council").modifier(adjp("local"))).the());

    assert_sentence(
        subject.predicate(predicate).sentence(),
        "The careful plan to repair the old bridge before the storm impressed the local council.",
    );
}

#[test]
fn we_did_not_expect_editor_to_read_manuscript_on_train() {
    let subject = pronoun_dp(Pronoun::We);

    let predicate = vp("expect")
        .past()
        .negative()
        .complement(
            dp(np("editor")
                .modifier(adjp("patient").modifier(advp("remarkably")))
                .complement(pp("with", dp(np("lantern")).a())))
            .the(),
        )
        .complement(
            vp("read")
                .to_infinitive()
                .complement(
                    dp(np("manuscript").modifier(adjp("long").modifier(advp("very")))).the(),
                )
                .adjunct(pp("on", dp(np("train")).the())),
        );

    assert_sentence(
        subject.predicate(predicate).sentence(),
        "We did not expect the remarkably patient editor with a lantern to read the very long manuscript on the train.",
    );
}

#[test]
fn alice_mailed_report_about_storm_to_office_near_harbor() {
    let subject = proper_name("Alice");

    let predicate = vp("mail")
        .past()
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
        ));

    assert_sentence(
        subject.predicate(predicate).sentence(),
        "Alice mailed the unusually detailed report about the storm over the northern coast to the quiet office near the harbor.",
    );
}

#[test]
fn old_machine_under_stairs_was_not_ready_to_move() {
    let subject = dp(np("machine")
        .modifier(adjp("old"))
        .complement(pp("under", dp(np("stairs")).the())))
    .the();

    let predicate = vp("be").past().negative().complement(
        adjp("ready").complement(
            vp("move")
                .to_infinitive()
                .adjunct(pp("into", dp(np("workshop")).the())),
        ),
    );

    assert_sentence(
        subject.predicate(predicate).sentence(),
        "The old machine under the stairs was not ready to move into the workshop.",
    );
}

#[test]
fn very_nearly_impossible_puzzle_confused_children_in_library() {
    let subject = dp(np("puzzle")
        .modifier(adjp("impossible").modifier(advp("nearly").modifier(advp("very"))))
        .complement(pp("from", dp(np("museum")).the())))
    .the();

    let predicate = vp("confuse")
        .past()
        .complement(
            dp(np("child")
                .plural()
                .modifier(adjp("patient").modifier(advp("extremely"))))
            .the(),
        )
        .adjunct(pp("in", dp(np("library")).the()));

    assert_sentence(
        subject.predicate(predicate).sentence(),
        "The very nearly impossible puzzle from the museum confused the extremely patient children in the library.",
    );
}

#[test]
fn they_discussed_mapping_cave_with_old_guide_after_meal() {
    let subject = pronoun_dp(Pronoun::They);

    let predicate = vp("discuss")
        .past()
        .complement(
            vp("map")
                .gerund_participle()
                .complement(
                    dp(np("cave")
                        .modifier(adjp("narrow"))
                        .complement(pp("under", dp(np("hill")).the())))
                    .the(),
                )
                .adjunct(pp("with", dp(np("guide").modifier(adjp("old"))).the())),
        )
        .adjunct(pp("after", dp(np("meal")).the()));

    assert_sentence(
        subject.predicate(predicate).sentence(),
        "They discussed mapping the narrow cave under the hill with the old guide after the meal.",
    );
}

#[test]
fn ambitious_attempt_to_persuade_pilot_to_land_plane_alarmed_team() {
    let subject = dp(np("attempt").modifier(adjp("ambitious")).complement(
        vp("persuade")
            .to_infinitive()
            .complement(dp(np("pilot").modifier(adjp("cautious"))).the())
            .complement(
                vp("land")
                    .to_infinitive()
                    .complement(dp(np("plane").modifier(adjp("damaged"))).the())
                    .adjunct(pp("near", dp(np("village")).the())),
            ),
    ))
    .the();

    let predicate = vp("alarm")
        .past()
        .complement(dp(np("team").modifier(adjp("rescue"))).the());

    assert_sentence(
        subject.predicate(predicate).sentence(),
        "The ambitious attempt to persuade the cautious pilot to land the damaged plane near the village alarmed the rescue team.",
    );
}
