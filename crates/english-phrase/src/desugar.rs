use crate::error::{RealizationError, RealizationResult};
use crate::internal::{
    ABar, AHead, AP, AdvBar, AdvHead, CBar, CHead, CP, DBar, DComplement, DHead, DP, NBar, NHead,
    NP, NegHead, NegVBar, PBar, PHead, PP, SilentDeterminer, TBar, THead, TP, VBar, VHead, VP,
    VPBar, XP,
};
use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, Clause, DeterminerHead, DeterminerPhrase, Phrase,
    PrepositionalPhrase, VerbForm, VerbPhrase,
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
        bar: DBar {
            head: DHead::Silent(SilentDeterminer::Trace),
            complement: DComplement::Trace,
        },
    }
}

pub(crate) fn lower_phrase(phrase: &Phrase) -> RealizationResult<XP> {
    match phrase {
        Phrase::DP(dp) => Ok(lower_dp(dp)?.into()),
        Phrase::VP(vp) => Ok(lower_verb_projection(vp, None)?.into()),
        Phrase::PP(pp) => Ok(lower_pp(pp)?.into()),
        Phrase::AdjP(ap) => Ok(lower_ap(ap)?.into()),
        Phrase::AdvP(advp) => Ok(lower_advp(advp)?.into()),
    }
}

pub(crate) fn lower_dp(phrase: &DeterminerPhrase) -> RealizationResult<DP> {
    let (d_head, n_head) = match phrase.head() {
        DeterminerHead::CommonNoun(noun) => (
            phrase
                .determiner_opt()
                .map(DHead::Overt)
                .unwrap_or(DHead::Silent(SilentDeterminer::Bare)),
            NHead::CommonNoun {
                entry: noun.clone(),
                number: phrase.number().clone(),
            },
        ),
        DeterminerHead::ProperName(name) => {
            if phrase.determiner_opt().is_some() {
                return Err(RealizationError::new(
                    "proper names do not take determiners in the fluent surface grammar",
                ));
            }

            (
                DHead::Silent(SilentDeterminer::ProperName),
                NHead::ProperName(name.clone()),
            )
        }
        DeterminerHead::Pronoun(pronoun) => {
            if phrase.determiner_opt().is_some() {
                return Err(RealizationError::new(
                    "pronouns do not take determiners in the fluent surface grammar",
                ));
            }

            (
                DHead::Silent(SilentDeterminer::Pronoun),
                NHead::Pronoun(*pronoun),
            )
        }
    };

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

    Ok(DP {
        bar: DBar {
            head: d_head,
            complement: DComplement::NP(Box::new(NP {
                left_adjuncts,
                bar: NBar {
                    head: n_head,
                    complements,
                },
            })),
        },
    })
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

fn lower_core_vp(phrase: &VerbPhrase, subject_trace: bool) -> RealizationResult<VP> {
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

pub(crate) fn lower_verb_projection(
    phrase: &VerbPhrase,
    subject: Option<&DeterminerPhrase>,
) -> RealizationResult<TP> {
    let subject_is_raised = subject.is_some();
    let lexical_vp = lower_core_vp(phrase, subject_is_raised)?;
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
        specifier: subject.map(lower_dp).transpose()?.map(Box::new),
        bar: TBar {
            head: t_head_from(phrase.form()),
            complement: Box::new(vp),
        },
    })
}

pub(crate) fn lower_clause(clause: &Clause) -> RealizationResult<CP> {
    Ok(CP {
        bar: CBar {
            head: CHead,
            complement: Box::new(lower_verb_projection(
                clause.predicate(),
                Some(clause.subject()),
            )?),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexical::{Determiner, Pronoun};
    use crate::syntax::{adjp, advp, dp, pp, vp};

    #[test]
    fn finite_clause_lowers_to_cp_tp_and_negated_vp() {
        let clause = lower_clause(
            &dp("child").determiner(Determiner::The).predicate(
                vp("eat")
                    .past()
                    .negative()
                    .complement(dp("apple").determiner(Determiner::The)),
            ),
        )
        .unwrap();

        assert!(clause.bar.complement.specifier.is_some());
        assert!(matches!(clause.bar.complement.bar.head, THead::Finite(_)));
        assert!(matches!(
            clause.bar.complement.bar.complement.bar,
            VPBar::Negated(_)
        ));

        let VPBar::Negated(neg) = &clause.bar.complement.bar.complement.bar else {
            panic!("expected negated VP");
        };
        assert!(matches!(neg.head, NegHead::Not));
        assert!(neg.complement.specifier.is_some());
    }

    #[test]
    fn dp_lowers_to_dp_dbar_np_with_adjuncts_and_complements() {
        let lowered = lower_dp(
            &dp("child")
                .determiner(Determiner::The)
                .modifier(adjp("happy").modifier(advp("very")))
                .complement(pp("with", dp("friend").determiner(Determiner::A))),
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
    fn pronouns_lower_to_silent_d_over_np() {
        let lowered = lower_dp(&crate::syntax::pronoun_dp(Pronoun::They)).unwrap();
        assert!(matches!(
            lowered.bar.head,
            DHead::Silent(SilentDeterminer::Pronoun)
        ));
    }
}
