use crate::error::{RealizationError, RealizationResult};
use crate::internal::{
    ABar, AHead, AP, AdvBar, AdvHead, DBar, DComplement, DHead, DP, NBar, NHead, NP, NegHead,
    NegVBar, PBar, PHead, PP, SilentDeterminer, TBar, THead, TP, VBar, VHead, VP, VPBar, XP,
};
use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, DeterminerPhrase, NounPhrase, Phrase, PrepositionalPhrase,
    TensePhrase, VerbForm, VerbPhrase,
};

fn t_head_from(form: VerbForm) -> THead {
    match form {
        VerbForm::Finite(tense) => THead::Finite(tense),
        VerbForm::BareInfinitive => THead::BareInfinitive,
        VerbForm::ToInfinitive => THead::ToInfinitive,
        VerbForm::GerundParticiple => THead::GerundParticiple,
        VerbForm::PastParticiple => THead::PastParticiple,
    }
}

fn trace_dp() -> DP {
    DP {
        specifier: None,
        bar: DBar {
            head: DHead::Silent(SilentDeterminer::Trace),
            complement: DComplement::Trace,
        },
    }
}

pub(crate) fn lower_phrase(phrase: &Phrase) -> RealizationResult<XP> {
    match phrase {
        Phrase::TP(tp) => Ok(lower_tense_phrase(tp)?.into()),
        Phrase::DP(dp) => Ok(lower_dp(dp)?.into()),
        Phrase::NP(np) => Ok(lower_np(np)?.into()),
        Phrase::VP(vp) => Ok(lower_verb_phrase(vp, false)?.into()),
        Phrase::PP(pp) => Ok(lower_pp(pp)?.into()),
        Phrase::AdjP(ap) => Ok(lower_ap(ap)?.into()),
        Phrase::AdvP(advp) => Ok(lower_advp(advp)?.into()),
    }
}

pub(crate) fn lower_np(phrase: &NounPhrase) -> RealizationResult<NP> {
    let left_adjuncts = phrase
        .modifiers()
        .iter()
        .map(|modifier| lower_phrase(modifier.as_ref()).map(Box::new))
        .collect::<RealizationResult<Vec<_>>>()?;

    let complements = phrase
        .complements()
        .iter()
        .map(|complement| lower_phrase(complement.as_ref()).map(Box::new))
        .collect::<RealizationResult<Vec<_>>>()?;

    Ok(NP {
        left_adjuncts,
        bar: NBar {
            head: NHead::CommonNoun {
                entry: phrase.head().clone(),
                number: phrase.number().clone(),
            },
            complements,
        },
    })
}

pub(crate) fn lower_dp(phrase: &DeterminerPhrase) -> RealizationResult<DP> {
    match phrase {
        DeterminerPhrase::BareNominal(nominal) => Ok(DP {
            specifier: None,
            bar: DBar {
                head: DHead::Silent(SilentDeterminer::Bare),
                complement: DComplement::NP(Box::new(lower_np(nominal)?)),
            },
        }),
        DeterminerPhrase::DeterminedNominal {
            determiner,
            nominal,
        } => Ok(DP {
            specifier: None,
            bar: DBar {
                head: DHead::Overt(*determiner),
                complement: DComplement::NP(Box::new(lower_np(nominal)?)),
            },
        }),
        DeterminerPhrase::PossessedNominal { possessor, nominal } => Ok(DP {
            specifier: Some(Box::new(lower_dp(possessor)?)),
            bar: DBar {
                head: DHead::Silent(SilentDeterminer::Bare),
                complement: DComplement::NP(Box::new(lower_np(nominal)?)),
            },
        }),
        DeterminerPhrase::ProperName(name) => Ok(DP {
            specifier: None,
            bar: DBar {
                head: DHead::Silent(SilentDeterminer::ProperName),
                complement: DComplement::NP(Box::new(NP {
                    left_adjuncts: Vec::new(),
                    bar: NBar {
                        head: NHead::ProperName(name.clone()),
                        complements: Vec::new(),
                    },
                })),
            },
        }),
        DeterminerPhrase::Pronoun { pronoun, reflexive } => Ok(DP {
            specifier: None,
            bar: DBar {
                head: DHead::Silent(SilentDeterminer::Pronoun),
                complement: DComplement::NP(Box::new(NP {
                    left_adjuncts: Vec::new(),
                    bar: NBar {
                        head: NHead::Pronoun {
                            entry: *pronoun,
                            reflexive: *reflexive,
                        },
                        complements: Vec::new(),
                    },
                })),
            },
        }),
    }
}

pub(crate) fn lower_ap(phrase: &AdjectivePhrase) -> RealizationResult<AP> {
    let specifier = phrase
        .modifier_opt()
        .map(|modifier| match modifier {
            Phrase::AdvP(advp) => lower_advp(advp).map(Box::new),
            _ => Err(RealizationError::new(
                "adjective phrase modifiers must lower to AdvP specifiers",
            )),
        })
        .transpose()?;

    let complements = phrase
        .complements()
        .iter()
        .map(|complement| lower_phrase(complement.as_ref()).map(Box::new))
        .collect::<RealizationResult<Vec<_>>>()?;

    Ok(AP {
        specifier,
        bar: ABar {
            head: AHead {
                entry: phrase.head().clone(),
            },
            complements,
        },
    })
}

pub(crate) fn lower_advp(phrase: &AdverbPhrase) -> RealizationResult<crate::internal::AdvP> {
    let specifier = phrase
        .modifier_opt()
        .map(|modifier| match modifier {
            Phrase::AdvP(advp) => lower_advp(advp).map(Box::new),
            _ => Err(RealizationError::new(
                "adverb phrase modifiers must lower to AdvP specifiers",
            )),
        })
        .transpose()?;

    let complements = phrase
        .complements()
        .iter()
        .map(|complement| lower_phrase(complement.as_ref()).map(Box::new))
        .collect::<RealizationResult<Vec<_>>>()?;

    Ok(crate::internal::AdvP {
        specifier,
        bar: AdvBar {
            head: AdvHead {
                entry: phrase.head().clone(),
            },
            complements,
        },
    })
}

pub(crate) fn lower_pp(phrase: &PrepositionalPhrase) -> RealizationResult<PP> {
    Ok(PP {
        bar: PBar {
            head: PHead {
                entry: phrase.head().clone(),
            },
            complement: Box::new(lower_phrase(phrase.complement())?),
        },
    })
}

pub(crate) fn lower_verb_phrase(phrase: &VerbPhrase, subject_trace: bool) -> RealizationResult<VP> {
    let complements = phrase
        .complements()
        .iter()
        .map(|complement| lower_phrase(complement.as_ref()).map(Box::new))
        .collect::<RealizationResult<Vec<_>>>()?;

    let adjuncts = phrase
        .adjuncts()
        .iter()
        .map(|adjunct| lower_phrase(adjunct.as_ref()).map(Box::new))
        .collect::<RealizationResult<Vec<_>>>()?;

    Ok(VP {
        specifier: subject_trace.then(|| Box::new(trace_dp())),
        bar: VPBar::Headed(VBar {
            head: VHead {
                entry: phrase.head().clone(),
            },
            complements,
            adjuncts,
        }),
    })
}

pub(crate) fn lower_tense_phrase(phrase: &TensePhrase) -> RealizationResult<TP> {
    let subject_is_raised = phrase.subject_opt().is_some();
    let lexical_vp = lower_verb_phrase(phrase.predicate(), subject_is_raised)?;
    let vp = if phrase.is_negative() {
        VP {
            specifier: None,
            bar: VPBar::Negated(NegVBar {
                head: NegHead::Not,
                complement: Box::new(lexical_vp),
            }),
        }
    } else {
        lexical_vp
    };

    Ok(TP {
        specifier: phrase
            .subject_opt()
            .map(lower_dp)
            .transpose()?
            .map(Box::new),
        bar: TBar {
            head: t_head_from(phrase.form()),
            complement: Box::new(vp),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexical::{Determiner, Pronoun};
    use crate::syntax::{Phrase, adjp, advp, dp, name, np, pp, tp, vp};

    #[test]
    fn finite_clause_lowers_to_tp_and_negated_vp() {
        let clause = lower_tense_phrase(
            &tp(vp("eat").complement(dp(np("apple")).the()))
                .past()
                .negative()
                .subject(dp(np("child")).the()),
        )
        .unwrap();

        assert!(clause.specifier.is_some());
        assert!(matches!(clause.bar.head, THead::Finite(_)));
        assert!(matches!(clause.bar.complement.bar, VPBar::Negated(_)));

        let VPBar::Negated(neg) = &clause.bar.complement.bar else {
            panic!("expected negated VP");
        };
        assert!(matches!(neg.head, NegHead::Not));
        assert!(neg.complement.specifier.is_some());
    }

    #[test]
    fn phrase_distinguishes_vp_from_tp() {
        let lexical =
            lower_phrase(&Phrase::from(vp("eat").complement(dp(np("apple")).the()))).unwrap();
        let clausal = lower_phrase(&Phrase::from(
            tp(vp("eat").complement(dp(np("apple")).the())).past(),
        ))
        .unwrap();

        assert!(matches!(lexical, XP::VP(_)));
        assert!(matches!(clausal, XP::TP(_)));
    }

    #[test]
    fn dp_lowers_to_dp_dbar_np_with_adjuncts_and_complements() {
        let lowered = lower_dp(
            &dp(np("child")
                .modifier(adjp("happy").modifier(advp("very")))
                .complement(pp("with", dp(np("friend")).indefinite())))
            .the(),
        )
        .unwrap();

        assert!(matches!(lowered.bar.head, DHead::Overt(Determiner::The)));

        let DComplement::NP(np) = lowered.bar.complement else {
            panic!("expected NP complement");
        };
        assert_eq!(np.left_adjuncts.len(), 1);
        assert_eq!(np.bar.complements.len(), 1);
        assert!(matches!(np.bar.head, NHead::CommonNoun { .. }));
    }

    #[test]
    fn possessors_lower_to_dp_specifiers() {
        let lowered = lower_dp(&dp(np("book")).possessor(dp(name("John")))).unwrap();

        assert!(lowered.specifier.is_some());
        assert!(matches!(
            lowered.bar.head,
            DHead::Silent(SilentDeterminer::Bare)
        ));
    }

    #[test]
    fn pronouns_lower_to_silent_d_over_np() {
        let lowered = lower_dp(&dp(Pronoun::They).into()).unwrap();
        assert!(matches!(
            lowered.bar.head,
            DHead::Silent(SilentDeterminer::Pronoun)
        ));
    }

    #[test]
    fn np_lowers_directly_to_np() {
        let lowered = lower_np(
            &np("child")
                .modifier(adjp("happy").modifier(advp("very")))
                .complement(pp("with", dp(np("friend")).indefinite())),
        )
        .unwrap();

        assert_eq!(lowered.left_adjuncts.len(), 1);
        assert_eq!(lowered.bar.complements.len(), 1);
        assert!(matches!(lowered.bar.head, NHead::CommonNoun { .. }));
    }
}
