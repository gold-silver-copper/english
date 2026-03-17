use english_phrase::*;
use english_phrase::syntax::TpForm;

fn assert_sentence<Form: TpForm>(clause: TensePhrase<Form>, expected: &str) {
    assert_eq!(
        clause.realize_with(RealizationOptions::sentence()),
        expected
    );
}

#[test]
fn remarkably_patient_editor_admired_herself_after_the_noisy_rehearsal() {
    let subject = dp(np("editor")
        .modifier(adjp("patient").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("office").modifier(adjp("northern"))).the(),
        )))
    .the();

    let clause = tp(vp("admire")
        .complement(dp(Pronoun::She).reflexive())
        .adjunct(pp(
            "after",
            dp(np("rehearsal").modifier(adjp("noisy"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The remarkably patient editor from the northern office admired herself after the noisy rehearsal near the harbor.",
    );
}

#[test]
fn remarkably_patient_editor_argued_with_her_after_the_unusually_long_meeting() {
    let subject = dp(np("editor")
        .modifier(adjp("patient").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("office").modifier(adjp("northern"))).the(),
        )))
    .the();

    let clause = tp(vp("argue")
        .complement(pp("with", dp(Pronoun::She)))
        .adjunct(pp(
            "after",
            dp(np("meeting").modifier(adjp("long").modifier(advp("unusually")))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The remarkably patient editor from the northern office argued with her after the unusually long meeting near the harbor.",
    );
}

#[test]
fn morgans_unusually_careful_map_of_the_narrow_cave_confused_the_rescue_team() {
    let subject = dp(np("map")
        .modifier(adjp("careful").modifier(advp("unusually")))
        .complement(pp(
            "of",
            dp(np("cave")
                .modifier(adjp("narrow"))
                .complement(pp("under", dp(np("hill").modifier(adjp("old"))).the())))
            .the(),
        )))
    .possessor(dp(name("Morgan")));

    let clause = tp(vp("confuse")
        .complement(dp(np("team").modifier(adjp("rescue"))).the())
        .adjunct(pp("after", dp(np("meal").modifier(adjp("long"))).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "Morgan's unusually careful map of the narrow cave under the old hill confused the rescue team after the long meal.",
    );
}

#[test]
fn patient_editor_showed_them_our_map_after_the_guide_described_it_to_us() {
    let subject = dp(np("editor")
        .modifier(adjp("patient").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("office").modifier(adjp("northern"))).the(),
        )))
    .the();

    let described_clause = cp(tp(vp("describe")
        .complement(dp(Pronoun::It))
        .adjunct(pp("to", dp(Pronoun::We)))
        .adjunct(pp("during", dp(np("meeting").modifier(adjp("long"))).the())))
    .past()
    .subject(dp(np("guide").modifier(adjp("patient"))).the()));

    let clause = tp(vp("show")
        .complement(dp(Pronoun::They))
        .complement(
            dp(np("map")
                .modifier(adjp("detailed").modifier(advp("unusually")))
                .complement(pp("of", dp(np("cave").modifier(adjp("narrow"))).the())))
            .possessor(dp(Pronoun::We)),
        )
        .adjunct(pp("after", described_clause)))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The remarkably patient editor from the northern office showed them our unusually detailed map of the narrow cave after the patient guide described it to us during the long meeting.",
    );
}

#[test]
fn this_unusually_careful_plan_to_repair_the_bridge_impressed_the_local_council() {
    let subject = dp(np("plan")
        .modifier(adjp("careful").modifier(advp("unusually")))
        .complement(
            tp(vp("repair")
                .complement(dp(np("bridge").modifier(adjp("old"))).the())
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        ))
    .this();

    let clause = tp(vp("impress")
        .complement(dp(np("council").modifier(adjp("local"))).the())
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "This unusually careful plan to repair the old bridge before the final inspection impressed the local council after the storm.",
    );
}

#[test]
fn those_remarkably_detailed_reports_about_the_northern_coast_alarmed_the_committee() {
    let subject = dp(np("report")
        .plural()
        .modifier(adjp("detailed").modifier(advp("remarkably")))
        .complement(pp(
            "about",
            dp(np("coast").modifier(adjp("northern"))).the(),
        )))
    .those();

    let clause = tp(vp("alarm")
        .complement(dp(np("committee").modifier(adjp("rescue"))).the())
        .adjunct(pp("during", dp(np("meeting").modifier(adjp("long"))).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "Those remarkably detailed reports about the northern coast alarmed the rescue committee during the long meeting.",
    );
}

#[test]
fn an_unusually_quiet_engine_under_the_old_staircase_alarmed_the_patient_mechanic() {
    let subject = dp(np("engine")
        .countable()
        .modifier(adjp("quiet").modifier(advp("unusually")))
        .complement(pp("under", dp(np("staircase").modifier(adjp("old"))).the())))
    .indefinite();

    let clause = tp(vp("alarm")
        .complement(dp(np("mechanic").modifier(adjp("patient"))).the())
        .adjunct(pp("after", dp(np("rehearsal")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "An unusually quiet engine under the old staircase alarmed the patient mechanic after the rehearsal.",
    );
}

#[test]
fn alice_mailed_the_unusually_detailed_report_to_the_quiet_office_near_the_harbor() {
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
        ))
        .adjunct(pp("after", dp(np("meeting").modifier(adjp("long"))).the())))
    .past()
    .subject(dp(name("Alice")));

    assert_sentence(
        clause,
        "Alice mailed the unusually detailed report about the storm over the northern coast to the quiet office near the harbor after the long meeting.",
    );
}

#[test]
fn we_did_not_expect_the_editor_with_a_lantern_to_read_the_manuscript_after_midnight() {
    let clause = tp(vp("expect")
        .complement(
            dp(np("editor")
                .modifier(adjp("patient").modifier(advp("remarkably")))
                .complement(pp("with", dp(np("lantern").countable()).indefinite())))
            .the(),
        )
        .complement(
            tp(vp("read")
                .complement(
                    dp(np("manuscript").modifier(adjp("long").modifier(advp("very")))).the(),
                )
                .adjunct(pp("on", dp(np("train")).the()))
                .adjunct(pp(
                    "after",
                    dp(np("rehearsal").modifier(adjp("late"))).the(),
                )))
            .to_infinitive(),
        ))
    .past()
    .negative()
    .subject(dp(Pronoun::We));

    assert_sentence(
        clause,
        "We did not expect the remarkably patient editor with a lantern to read the very long manuscript on the train after the late rehearsal.",
    );
}

#[test]
fn ambitious_attempt_to_persuade_the_pilot_to_repair_the_engine_alarmed_the_council() {
    let subject = dp(np("attempt")
        .modifier(adjp("ambitious").modifier(advp("unusually")))
        .complement(
            tp(vp("persuade")
                .complement(dp(np("pilot").modifier(adjp("cautious"))).the())
                .complement(
                    tp(vp("repair")
                        .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                        .adjunct(pp(
                            "before",
                            dp(np("inspection").modifier(adjp("final"))).the(),
                        )))
                    .to_infinitive(),
                ))
            .to_infinitive(),
        ))
    .the();

    let clause = tp(vp("alarm")
        .complement(dp(np("council").modifier(adjp("local"))).the())
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The unusually ambitious attempt to persuade the cautious pilot to repair the damaged engine before the final inspection alarmed the local council after the storm.",
    );
}

#[test]
fn patient_committee_discussed_mapping_the_cave_with_the_experienced_guide() {
    let subject = dp(np("committee")
        .modifier(adjp("patient"))
        .complement(pp("from", dp(np("council").modifier(adjp("local"))).the())))
    .the();

    let clause = tp(vp("discuss")
        .complement(
            tp(vp("map")
                .complement(
                    dp(np("cave")
                        .modifier(adjp("narrow"))
                        .complement(pp("under", dp(np("hill").modifier(adjp("old"))).the())))
                    .the(),
                )
                .adjunct(pp(
                    "with",
                    dp(np("guide").modifier(adjp("experienced"))).the(),
                )))
            .gerund_participle(),
        )
        .adjunct(pp("after", dp(np("meal").modifier(adjp("long"))).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The patient committee from the local council discussed mapping the narrow cave under the old hill with the experienced guide after the long meal.",
    );
}

#[test]
fn these_remarkably_patient_mechanics_were_ready_to_move_the_engine_before_the_inspection() {
    let subject = dp(np("mechanic")
        .plural()
        .modifier(adjp("patient").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("workshop").modifier(adjp("northern"))).the(),
        )))
    .these();

    let clause = tp(vp("be").complement(
        adjp("ready").complement(
            tp(vp("move")
                .complement(dp(np("engine").modifier(adjp("damaged"))).the())
                .adjunct(pp("into", dp(np("garage").modifier(adjp("quiet"))).the()))
                .adjunct(pp(
                    "before",
                    dp(np("inspection").modifier(adjp("final"))).the(),
                )))
            .to_infinitive(),
        ),
    ))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "These remarkably patient mechanics from the northern workshop were ready to move the damaged engine into the quiet garage before the final inspection.",
    );
}

#[test]
fn that_unusually_quiet_engine_was_not_damaged_after_the_violent_storm() {
    let subject = dp(np("engine")
        .modifier(adjp("quiet").modifier(advp("unusually")))
        .complement(pp("under", dp(np("staircase").modifier(adjp("old"))).the())))
    .that();

    let clause = tp(vp("be")
        .complement(tp(vp("damage")).past_participle())
        .adjunct(pp("after", dp(np("storm").modifier(adjp("violent"))).the()))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .past()
    .negative()
    .subject(subject);

    assert_sentence(
        clause,
        "That unusually quiet engine under the old staircase was not damaged after the violent storm near the harbor.",
    );
}

#[test]
fn unusually_generous_curator_gave_the_editor_the_report_after_the_rehearsal() {
    let subject = dp(np("curator")
        .modifier(adjp("generous").modifier(advp("unusually")))
        .complement(pp("from", dp(np("museum").modifier(adjp("coastal"))).the())))
    .the();

    let clause = tp(vp("give")
        .complement(dp(np("editor").modifier(adjp("patient").modifier(advp("remarkably")))).the())
        .complement(dp(np("report").modifier(adjp("detailed").modifier(advp("very")))).the())
        .adjunct(pp(
            "after",
            dp(np("rehearsal").modifier(adjp("long"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The unusually generous curator from the coastal museum gave the remarkably patient editor the very detailed report after the long rehearsal near the harbor.",
    );
}

#[test]
fn careful_mechanic_moved_the_engine_from_the_old_garage_to_the_quiet_yard() {
    let subject = dp(np("mechanic").modifier(adjp("careful")).complement(pp(
        "from",
        dp(np("workshop").modifier(adjp("northern"))).the(),
    )))
    .the();

    let clause = tp(vp("move")
        .complement(dp(np("engine").modifier(adjp("damaged"))).the())
        .complement(pp("from", dp(np("garage").modifier(adjp("old"))).the()))
        .complement(pp("to", dp(np("yard").modifier(adjp("quiet"))).the()))
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The careful mechanic from the northern workshop moved the damaged engine from the old garage to the quiet yard after the storm.",
    );
}

#[test]
fn patient_editor_read_the_manuscript_remarkably_carefully_during_the_late_rehearsal() {
    let subject = dp(np("editor")
        .modifier(adjp("patient").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("office").modifier(adjp("northern"))).the(),
        )))
    .the();

    let clause = tp(vp("read")
        .complement(dp(np("manuscript").modifier(adjp("long").modifier(advp("very")))).the())
        .adjunct(
            advp("carefully")
                .modifier(advp("remarkably"))
                .complement(pp(
                    "during",
                    dp(np("rehearsal").modifier(adjp("late"))).the(),
                )),
        )
        .adjunct(pp("after", dp(np("storm")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The remarkably patient editor from the northern office read the very long manuscript remarkably carefully during the late rehearsal after the storm.",
    );
}

#[test]
fn cautious_pilot_relaxed_after_the_rescue_team_arrived_with_the_old_guide() {
    let subject = dp(np("pilot").modifier(adjp("cautious")).complement(pp(
        "from",
        dp(np("village").modifier(adjp("northern"))).the(),
    )))
    .the();

    let arrived_clause = cp(tp(vp("arrive")
        .adjunct(pp("with", dp(np("guide").modifier(adjp("old"))).the()))
        .adjunct(pp(
            "before",
            dp(np("inspection").modifier(adjp("final"))).the(),
        )))
    .past()
    .subject(dp(np("team").modifier(adjp("rescue"))).the()));

    let clause = tp(vp("relax").adjunct(pp("after", arrived_clause)))
        .past()
        .subject(subject);

    assert_sentence(
        clause,
        "The cautious pilot from the northern village relaxed after the rescue team arrived with the old guide before the final inspection.",
    );
}

#[test]
fn i_told_you_he_admired_her_after_the_long_rehearsal_near_the_harbor() {
    let embedded_clause = cp(tp(vp("admire")
        .complement(dp(Pronoun::She))
        .adjunct(pp(
            "after",
            dp(np("rehearsal").modifier(adjp("long"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .past()
    .subject(dp(Pronoun::He)));

    let clause = tp(vp("tell")
        .complement(dp(Pronoun::You))
        .complement(embedded_clause))
    .past()
    .subject(dp(Pronoun::I));

    assert_sentence(
        clause,
        "I told you he admired her after the long rehearsal near the harbor.",
    );
}

#[test]
fn cautious_guide_walked_from_near_the_river_to_the_quiet_harbor() {
    let subject = dp(np("guide").modifier(adjp("cautious")).complement(pp(
        "from",
        dp(np("village").modifier(adjp("northern"))).the(),
    )))
    .the();

    let clause = tp(vp("walk")
        .complement(pp("from", pp("near", dp(np("river")).the())))
        .adjunct(pp("to", dp(np("harbor").modifier(adjp("quiet"))).the()))
        .adjunct(pp("after", dp(np("storm").modifier(adjp("violent"))).the()))
        .adjunct(pp(
            "before",
            dp(np("inspection").modifier(adjp("final"))).the(),
        )))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The cautious guide from the northern village walked from near the river to the quiet harbor after the violent storm before the final inspection.",
    );
}

#[test]
fn remarkably_careful_guide_was_deeply_proud_of_them_after_the_difficult_rescue() {
    let subject = dp(np("guide")
        .modifier(adjp("careful").modifier(advp("remarkably")))
        .complement(pp(
            "from",
            dp(np("village").modifier(adjp("coastal"))).the(),
        )))
    .the();

    let clause = tp(vp("be")
        .complement(
            adjp("proud")
                .modifier(advp("deeply"))
                .complement(pp("of", dp(Pronoun::They))),
        )
        .adjunct(pp(
            "after",
            dp(np("rescue").modifier(adjp("difficult"))).the(),
        ))
        .adjunct(pp("near", dp(np("harbor")).the())))
    .past()
    .subject(subject);

    assert_sentence(
        clause,
        "The remarkably careful guide from the coastal village was deeply proud of them after the difficult rescue near the harbor.",
    );
}
