use english_phrase::*;

fn assert_sentence(subject: DeterminerPhrase, predicate: VerbPhrase, expected: &str) {
    assert_eq!(realize_sentence(subject, predicate).unwrap(), expected);
}

#[test]
fn careful_plan_to_repair_bridge_impressed_council() {
    let subject = dp("plan")
        .determiner(Determiner::The)
        .modifier(adjp("careful"))
        .complement(
            vp("repair")
                .to_infinitive()
                .complement(
                    dp("bridge")
                        .determiner(Determiner::The)
                        .modifier(adjp("old")),
                )
                .adjunct(pp("before", dp("storm").determiner(Determiner::The))),
        );

    let predicate = vp("impress").past().complement(
        dp("council")
            .determiner(Determiner::The)
            .modifier(adjp("local")),
    );

    assert_sentence(
        subject,
        predicate,
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
            dp("editor")
                .determiner(Determiner::The)
                .modifier(adjp("patient").modifier(advp("remarkably")))
                .complement(pp("with", dp("lantern").determiner(Determiner::A))),
        )
        .complement(
            vp("read")
                .to_infinitive()
                .complement(
                    dp("manuscript")
                        .determiner(Determiner::The)
                        .modifier(adjp("long").modifier(advp("very"))),
                )
                .adjunct(pp("on", dp("train").determiner(Determiner::The))),
        );

    assert_sentence(
        subject,
        predicate,
        "We did not expect the remarkably patient editor with a lantern to read the very long manuscript on the train.",
    );
}

#[test]
fn alice_mailed_report_about_storm_to_office_near_harbor() {
    let subject = proper_name("Alice");

    let predicate = vp("mail")
        .past()
        .complement(
            dp("report")
                .determiner(Determiner::The)
                .modifier(adjp("detailed").modifier(advp("unusually")))
                .complement(pp(
                    "about",
                    dp("storm").determiner(Determiner::The).complement(pp(
                        "over",
                        dp("coast")
                            .determiner(Determiner::The)
                            .modifier(adjp("northern")),
                    )),
                )),
        )
        .adjunct(pp(
            "to",
            dp("office")
                .determiner(Determiner::The)
                .modifier(adjp("quiet"))
                .complement(pp("near", dp("harbor").determiner(Determiner::The))),
        ));

    assert_sentence(
        subject,
        predicate,
        "Alice mailed the unusually detailed report about the storm over the northern coast to the quiet office near the harbor.",
    );
}

#[test]
fn old_machine_under_stairs_was_not_ready_to_move() {
    let subject = dp("machine")
        .determiner(Determiner::The)
        .modifier(adjp("old"))
        .complement(pp("under", dp("stairs").determiner(Determiner::The)));

    let predicate = vp("be").past().negative().complement(
        adjp("ready").complement(
            vp("move")
                .to_infinitive()
                .adjunct(pp("into", dp("workshop").determiner(Determiner::The))),
        ),
    );

    assert_sentence(
        subject,
        predicate,
        "The old machine under the stairs was not ready to move into the workshop.",
    );
}

#[test]
fn very_nearly_impossible_puzzle_confused_children_in_library() {
    let subject = dp("puzzle")
        .determiner(Determiner::The)
        .modifier(adjp("impossible").modifier(advp("nearly").modifier(advp("very"))))
        .complement(pp("from", dp("museum").determiner(Determiner::The)));

    let predicate = vp("confuse")
        .past()
        .complement(
            dp("child")
                .determiner(Determiner::The)
                .plural()
                .modifier(adjp("patient").modifier(advp("extremely"))),
        )
        .adjunct(pp("in", dp("library").determiner(Determiner::The)));

    assert_sentence(
        subject,
        predicate,
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
                    dp("cave")
                        .determiner(Determiner::The)
                        .modifier(adjp("narrow"))
                        .complement(pp("under", dp("hill").determiner(Determiner::The))),
                )
                .adjunct(pp(
                    "with",
                    dp("guide")
                        .determiner(Determiner::The)
                        .modifier(adjp("old")),
                )),
        )
        .adjunct(pp("after", dp("meal").determiner(Determiner::The)));

    assert_sentence(
        subject,
        predicate,
        "They discussed mapping the narrow cave under the hill with the old guide after the meal.",
    );
}

#[test]
fn ambitious_attempt_to_persuade_pilot_to_land_plane_alarmed_team() {
    let subject = dp("attempt")
        .determiner(Determiner::The)
        .modifier(adjp("ambitious"))
        .complement(
            vp("persuade")
                .to_infinitive()
                .complement(
                    dp("pilot")
                        .determiner(Determiner::The)
                        .modifier(adjp("cautious")),
                )
                .complement(
                    vp("land")
                        .to_infinitive()
                        .complement(
                            dp("plane")
                                .determiner(Determiner::The)
                                .modifier(adjp("damaged")),
                        )
                        .adjunct(pp("near", dp("village").determiner(Determiner::The))),
                ),
        );

    let predicate = vp("alarm").past().complement(
        dp("team")
            .determiner(Determiner::The)
            .modifier(adjp("rescue")),
    );

    assert_sentence(
        subject,
        predicate,
        "The ambitious attempt to persuade the cautious pilot to land the damaged plane near the village alarmed the rescue team.",
    );
}
