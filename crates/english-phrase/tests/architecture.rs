use english_phrase::*;

#[test]
fn builders_derive_into_pure_syntax_without_render_methods() {
    let subject = DeterminerPhraseBuilder::common_noun(NounEntry::animate("child"))
        .determiner(Determiner::The)
        .plural()
        .build()
        .unwrap();

    let predicate = VerbPhraseBuilder::new(VerbEntry::transitive("eat"))
        .direct_object(
            DeterminerPhraseBuilder::common_noun("apple")
                .determiner(Determiner::The)
                .build()
                .unwrap(),
        )
        .build();

    let clause = TensePhraseBuilder::new(subject, predicate)
        .past()
        .build()
        .unwrap();

    assert!(matches!(clause.predicate, VerbalProjection::Voice(_)));
    assert_eq!(
        realize_tense_phrase(&clause).unwrap(),
        "the children ate the apple"
    );
}

#[test]
fn canonical_dp_variants_cover_projection_pronoun_proper_name_reflexive_and_gap() {
    let projected = DeterminerPhraseBuilder::common_noun("teacher")
        .determiner(Determiner::The)
        .build()
        .unwrap();
    let pronoun = DeterminerPhraseBuilder::pronoun(Pronoun::They)
        .build()
        .unwrap();
    let proper_name = DeterminerPhraseBuilder::proper_name("Alice")
        .build()
        .unwrap();
    let reflexive = DeterminerPhraseBuilder::reflexive_from(&proper_name)
        .build()
        .unwrap();
    let gap = DeterminerPhraseBuilder::gap(
        GapDependency::new(DependencyRole::DirectObject),
        DpSemantics::new(Person::Third, Number::Singular).with_binding_key(BindingKey(7)),
    )
    .build()
    .unwrap();

    assert!(matches!(
        projected.kind,
        DeterminerPhraseKind::Projection(_)
    ));
    assert!(matches!(
        pronoun.kind,
        DeterminerPhraseKind::BarePronoun { .. }
    ));
    assert!(matches!(
        proper_name.kind,
        DeterminerPhraseKind::Projection(_)
    ));
    assert!(matches!(
        reflexive.kind,
        DeterminerPhraseKind::ReflexivePronoun { .. }
    ));
    assert!(matches!(gap.kind, DeterminerPhraseKind::Gap { .. }));
}

#[test]
fn lexical_selection_is_checked_during_derivation() {
    let subject = DeterminerPhraseBuilder::pronoun(Pronoun::They)
        .build()
        .unwrap();
    let predicate = VerbPhraseBuilder::new(VerbEntry::intransitive("sleep"))
        .direct_object(
            DeterminerPhraseBuilder::common_noun("bed")
                .determiner(Determiner::The)
                .build()
                .unwrap(),
        )
        .build();

    let error = TensePhraseBuilder::new(subject, predicate)
        .build()
        .unwrap_err();
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.code == "forbidden-direct-object")
    );
}

#[test]
fn passive_and_reflexive_are_derivation_operations_not_ast_methods() {
    let subject = DeterminerPhraseBuilder::proper_name("Alice")
        .feminine()
        .binding_key(BindingKey(3))
        .build()
        .unwrap();
    let object = DeterminerPhraseBuilder::proper_name("Alice")
        .feminine()
        .binding_key(BindingKey(3))
        .build()
        .unwrap();

    let passive = TensePhraseBuilder::new(
        subject.clone(),
        VerbPhraseBuilder::new(VerbEntry::transitive("praise"))
            .direct_object(
                DeterminerPhraseBuilder::common_noun("child")
                    .determiner(Determiner::The)
                    .build()
                    .unwrap(),
            )
            .build(),
    )
    .past()
    .passive()
    .build()
    .unwrap();

    let reflexive = TensePhraseBuilder::new(
        subject,
        VerbPhraseBuilder::new(VerbEntry::transitive("admire"))
            .direct_object(object)
            .build(),
    )
    .past()
    .reflexive()
    .build()
    .unwrap();

    assert_eq!(
        realize_tense_phrase(&passive).unwrap(),
        "the child was praised by Alice"
    );
    assert_eq!(
        realize_tense_phrase(&reflexive).unwrap(),
        "Alice admired herself"
    );
}

#[test]
fn explicit_verbal_spine_realizes_modal_negative_perfect_progressive_and_voice() {
    let subject = DeterminerPhraseBuilder::common_noun("child")
        .determiner(Determiner::The)
        .plural()
        .build()
        .unwrap();
    let predicate = VerbPhraseBuilder::new(VerbEntry::transitive("praise"))
        .direct_object(
            DeterminerPhraseBuilder::common_noun("teacher")
                .determiner(Determiner::The)
                .build()
                .unwrap(),
        )
        .build();

    let clause = TensePhraseBuilder::new(subject, predicate)
        .modal(Modal::Would)
        .negative()
        .perfect()
        .progressive()
        .passive()
        .build()
        .unwrap();

    assert!(matches!(clause.predicate, VerbalProjection::Modal(_)));
    assert_eq!(
        realize_tense_phrase(&clause).unwrap(),
        "the teacher would not have been being praised by the children"
    );
}

#[test]
fn nonfinite_clauses_are_realized_through_the_same_pipeline() {
    let clause = NonFiniteClauseBuilder::new(
        VerbPhraseBuilder::new(VerbEntry::transitive("eat"))
            .direct_object(
                DeterminerPhraseBuilder::common_noun("apple")
                    .determiner(Determiner::The)
                    .build()
                    .unwrap(),
            )
            .build(),
    )
    .to_infinitive()
    .negative()
    .perfect()
    .build()
    .unwrap();

    assert_eq!(
        realize_non_finite_clause(&clause).unwrap(),
        "not to have eaten the apple"
    );
}

#[test]
fn builders_and_realizer_support_structured_cp_and_relative_material() {
    let gap = DeterminerPhraseBuilder::gap(
        GapDependency::new(DependencyRole::Subject).with_binder(BindingKey(11)),
        DpSemantics::new(Person::Third, Number::Singular)
            .with_binding_key(BindingKey(11))
            .with_animacy(Animacy::Animate),
    )
    .build()
    .unwrap();

    let relative_tp = TensePhraseBuilder::new(
        gap,
        VerbPhraseBuilder::new(VerbEntry::intransitive("arrive")).build(),
    )
    .past()
    .build()
    .unwrap();

    let relative = RelativeClause {
        marker: RelativeMarker::Who,
        clause: Box::new(relative_tp),
    };

    let dp = DeterminerPhraseBuilder::common_noun("child")
        .determiner(Determiner::The)
        .postmodifier(relative)
        .build()
        .unwrap();

    let matrix = TensePhraseBuilder::new(
        dp,
        VerbPhraseBuilder::new(VerbEntry::intransitive("wait")).build(),
    )
    .past()
    .build()
    .unwrap();

    assert_eq!(
        realize_tense_phrase(&matrix).unwrap(),
        "the child who arrived waited"
    );
}

#[test]
fn sentence_realization_is_fallible_but_convenient() {
    let clause = Clause {
        tense_phrase: TensePhraseBuilder::new(
            DeterminerPhraseBuilder::pronoun(Pronoun::We)
                .build()
                .unwrap(),
            VerbPhraseBuilder::new(VerbEntry::intransitive("arrive")).build(),
        )
        .past()
        .build()
        .unwrap(),
    };

    let sentence = SentenceBuilder::new(clause).capitalize().period().build();

    assert_eq!(realize_sentence(&sentence).unwrap(), "We arrived.");
}
