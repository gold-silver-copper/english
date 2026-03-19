use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, AgreementMarker, ApComplement, Complementizer,
    ContentClause, CpForce, DeterminerPhrase, DeterminerPhraseKind, DynamicDeterminerPhrase,
    IntoDynamicDeterminerPhrase, NominalAgreementMarker, NominalCountabilityMarker,
    NominalDeterminerPhrase, NominalNumberMarker, NounPhrase, NounPhraseData, NpAdjunct,
    NpComplement, NpModifier, PpComplement, PredicateGap, PrepositionalPhrase,
    PronominalDeterminerPhrase, RelativeClause, RelativeTpGap, Relativizer, TensePhrase, TpForm,
    TpGap, VerbForm, VerbPhrase, VpAdjunct, VpArgumentSlot, VpComplement,
};
use english::{Number, Person};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeDeterminerPhrase {
    pub(super) kind: RuntimeDeterminerPhraseKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeDeterminerPhraseKind {
    BareNominal(Box<RuntimeNounPhrase>),
    DeterminedNominal {
        determiner: Determiner,
        nominal: Box<RuntimeNounPhrase>,
    },
    PossessedNominal {
        possessor: Box<RuntimeDeterminerPhrase>,
        nominal: Box<RuntimeNounPhrase>,
    },
    ProperName(String),
    Pronoun {
        pronoun: Pronoun,
        reflexive: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeNounPhrase {
    pub(super) head: NounEntry,
    pub(super) number: Number,
    pub(super) modifiers: Vec<RuntimeNpModifier>,
    pub(super) complements: Vec<RuntimeNpComplement>,
    pub(super) adjuncts: Vec<RuntimeNpAdjunct>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeNpModifier {
    Adj(RuntimeAdjectivePhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeNpComplement {
    PP(RuntimePrepositionalPhrase),
    TP(RuntimeTensePhrase),
    ContentClause(RuntimeContentClause),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeNpAdjunct {
    PP(RuntimePrepositionalPhrase),
    RelativeClause(RuntimeRelativeClause),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeAdjectivePhrase {
    pub(super) modifier: Option<Box<RuntimeAdverbPhrase>>,
    pub(super) head: AdjectiveEntry,
    pub(super) complements: Vec<RuntimeApComplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeApComplement {
    PP(RuntimePrepositionalPhrase),
    TP(RuntimeTensePhrase),
    ContentClause(RuntimeContentClause),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeAdverbPhrase {
    pub(super) modifier: Option<Box<RuntimeAdverbPhrase>>,
    pub(super) head: AdverbEntry,
    pub(super) complements: Vec<RuntimeAdvpComplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeAdvpComplement {
    PP(RuntimePrepositionalPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimePrepositionalPhrase {
    pub(super) head: PrepositionEntry,
    pub(super) complement: Box<RuntimePpComplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimePpComplement {
    DP(RuntimeDeterminerPhrase),
    PP(RuntimePrepositionalPhrase),
    TP(RuntimeTensePhrase),
    ContentClause(RuntimeContentClause),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeVerbPhrase {
    pub(super) head: VerbEntry,
    pub(super) complements: Vec<RuntimeVpComplement>,
    pub(super) adjuncts: Vec<RuntimeVpAdjunct>,
    pub(super) has_object_gap: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeVpComplement {
    DP(RuntimeDeterminerPhrase),
    PP(RuntimePrepositionalPhrase),
    AP(RuntimeAdjectivePhrase),
    ContentClause(RuntimeContentClause),
    TP(RuntimeTensePhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RuntimeVpAdjunct {
    PP(RuntimePrepositionalPhrase),
    AdvP(RuntimeAdverbPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeTensePhrase {
    pub(super) subject: Option<RuntimeDeterminerPhrase>,
    pub(super) predicate: RuntimeVerbPhrase,
    pub(super) form: VerbForm,
    pub(super) negative: bool,
    pub(super) agreement: Option<(Person, Number)>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeContentClause {
    pub(super) head: Complementizer,
    pub(super) complement: Box<RuntimeTensePhrase>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RuntimeRelativeClause {
    pub(super) head: Relativizer,
    pub(super) complement: Box<RuntimeTensePhrase>,
}

pub(super) trait LowerRuntime {
    type Runtime;

    fn lower_runtime(&self) -> Self::Runtime;
}

fn agreement_from_dp(dp: &DynamicDeterminerPhrase) -> (Person, Number) {
    match &dp.kind {
        DeterminerPhraseKind::BareNominal(nominal)
        | DeterminerPhraseKind::DeterminedNominal { nominal, .. }
        | DeterminerPhraseKind::PossessedNominal { nominal, .. } => (Person::Third, nominal.number),
        DeterminerPhraseKind::ProperName(_) => (Person::Third, Number::Singular),
        DeterminerPhraseKind::Pronoun { pronoun, .. } => (pronoun.person(), pronoun.number()),
    }
}

fn lower_np_data(np: &NounPhraseData) -> RuntimeNounPhrase {
    RuntimeNounPhrase {
        head: np.head.clone(),
        number: np.number,
        modifiers: np.modifiers.iter().map(lower_np_modifier).collect(),
        complements: np.complements.iter().map(lower_np_complement).collect(),
        adjuncts: np.adjuncts.iter().map(lower_np_adjunct).collect(),
    }
}

fn lower_np_modifier(modifier: &NpModifier) -> RuntimeNpModifier {
    match modifier {
        NpModifier::Adj(ap) => RuntimeNpModifier::Adj(ap.lower_runtime()),
    }
}

fn lower_np_complement(complement: &NpComplement) -> RuntimeNpComplement {
    match complement {
        NpComplement::PP(pp) => RuntimeNpComplement::PP(pp.lower_runtime()),
        NpComplement::ToInf(tp) => RuntimeNpComplement::TP(tp.lower_runtime()),
        NpComplement::CP(cp) => RuntimeNpComplement::ContentClause(cp.lower_runtime()),
    }
}

fn lower_np_adjunct(adjunct: &NpAdjunct) -> RuntimeNpAdjunct {
    match adjunct {
        NpAdjunct::PP(pp) => RuntimeNpAdjunct::PP(pp.lower_runtime()),
        NpAdjunct::RelativeObject(relative) => {
            RuntimeNpAdjunct::RelativeClause(relative.lower_runtime())
        }
        NpAdjunct::RelativeSubjectSingular(relative) => {
            RuntimeNpAdjunct::RelativeClause(relative.lower_runtime())
        }
        NpAdjunct::RelativeSubjectPlural(relative) => {
            RuntimeNpAdjunct::RelativeClause(relative.lower_runtime())
        }
    }
}

fn lower_ap_complement(complement: &ApComplement) -> RuntimeApComplement {
    match complement {
        ApComplement::PP(pp) => RuntimeApComplement::PP(pp.lower_runtime()),
        ApComplement::ToInf(tp) => RuntimeApComplement::TP(tp.lower_runtime()),
        ApComplement::CP(cp) => RuntimeApComplement::ContentClause(cp.lower_runtime()),
    }
}

fn lower_advp_complement(complement: &AdvpComplement) -> RuntimeAdvpComplement {
    match complement {
        AdvpComplement::PP(pp) => RuntimeAdvpComplement::PP(pp.lower_runtime()),
    }
}

fn lower_pp_complement(complement: &PpComplement) -> RuntimePpComplement {
    match complement {
        PpComplement::DP(dp) => RuntimePpComplement::DP(dp.lower_runtime()),
        PpComplement::PP(pp) => RuntimePpComplement::PP(pp.lower_runtime()),
        PpComplement::Gerund(tp) => RuntimePpComplement::TP(tp.lower_runtime()),
        PpComplement::CP(cp) => RuntimePpComplement::ContentClause(cp.lower_runtime()),
    }
}

fn lower_vp_complement(complement: &VpComplement) -> RuntimeVpComplement {
    match complement {
        VpComplement::DP(dp) => RuntimeVpComplement::DP(dp.lower_runtime()),
        VpComplement::PP(pp) => RuntimeVpComplement::PP(pp.lower_runtime()),
        VpComplement::AP(ap) => RuntimeVpComplement::AP(ap.lower_runtime()),
        VpComplement::CP(cp) => RuntimeVpComplement::ContentClause(cp.lower_runtime()),
        VpComplement::BareInf(tp) => RuntimeVpComplement::TP(tp.lower_runtime()),
        VpComplement::ToInf(tp) => RuntimeVpComplement::TP(tp.lower_runtime()),
        VpComplement::Gerund(tp) => RuntimeVpComplement::TP(tp.lower_runtime()),
        VpComplement::PastParticiple(tp) => RuntimeVpComplement::TP(tp.lower_runtime()),
    }
}

fn lower_vp_adjunct(adjunct: &VpAdjunct) -> RuntimeVpAdjunct {
    match adjunct {
        VpAdjunct::PP(pp) => RuntimeVpAdjunct::PP(pp.lower_runtime()),
        VpAdjunct::AdvP(advp) => RuntimeVpAdjunct::AdvP(advp.lower_runtime()),
    }
}

impl<A: AgreementMarker> LowerRuntime for DeterminerPhrase<A> {
    type Runtime = RuntimeDeterminerPhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        RuntimeDeterminerPhrase {
            kind: match &self.kind {
                DeterminerPhraseKind::BareNominal(nominal) => {
                    RuntimeDeterminerPhraseKind::BareNominal(Box::new(lower_np_data(nominal)))
                }
                DeterminerPhraseKind::DeterminedNominal {
                    determiner,
                    nominal,
                } => RuntimeDeterminerPhraseKind::DeterminedNominal {
                    determiner: *determiner,
                    nominal: Box::new(lower_np_data(nominal)),
                },
                DeterminerPhraseKind::PossessedNominal { possessor, nominal } => {
                    RuntimeDeterminerPhraseKind::PossessedNominal {
                        possessor: Box::new(possessor.lower_runtime()),
                        nominal: Box::new(lower_np_data(nominal)),
                    }
                }
                DeterminerPhraseKind::ProperName(name) => {
                    RuntimeDeterminerPhraseKind::ProperName(name.clone())
                }
                DeterminerPhraseKind::Pronoun { pronoun, reflexive } => {
                    RuntimeDeterminerPhraseKind::Pronoun {
                        pronoun: *pronoun,
                        reflexive: *reflexive,
                    }
                }
            },
        }
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> LowerRuntime
    for NominalDeterminerPhrase<N, C>
where
    N: NominalAgreementMarker,
{
    type Runtime = RuntimeDeterminerPhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        self.clone().into_dynamic_dp().lower_runtime()
    }
}

impl LowerRuntime for PronominalDeterminerPhrase {
    type Runtime = RuntimeDeterminerPhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        (*self).into_dynamic_dp().lower_runtime()
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> LowerRuntime for NounPhrase<N, C> {
    type Runtime = RuntimeNounPhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        lower_np_data(&self.data)
    }
}

impl LowerRuntime for AdjectivePhrase {
    type Runtime = RuntimeAdjectivePhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        RuntimeAdjectivePhrase {
            modifier: self
                .modifier_opt()
                .map(|modifier| Box::new(modifier.lower_runtime())),
            head: self.head().clone(),
            complements: self.complements().iter().map(lower_ap_complement).collect(),
        }
    }
}

impl LowerRuntime for AdverbPhrase {
    type Runtime = RuntimeAdverbPhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        RuntimeAdverbPhrase {
            modifier: self
                .modifier_opt()
                .map(|modifier| Box::new(modifier.lower_runtime())),
            head: self.head().clone(),
            complements: self
                .complements()
                .iter()
                .map(lower_advp_complement)
                .collect(),
        }
    }
}

impl LowerRuntime for PrepositionalPhrase {
    type Runtime = RuntimePrepositionalPhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        RuntimePrepositionalPhrase {
            head: self.head().clone(),
            complement: Box::new(lower_pp_complement(self.complement())),
        }
    }
}

impl<G: PredicateGap> LowerRuntime for VerbPhrase<G> {
    type Runtime = RuntimeVerbPhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        let mut complements = Vec::new();
        let mut has_object_gap = false;

        for slot in self.argument_slots() {
            match slot {
                VpArgumentSlot::Complement(complement) => {
                    complements.push(lower_vp_complement(complement));
                }
                VpArgumentSlot::GapObject => {
                    has_object_gap = true;
                }
            }
        }

        RuntimeVerbPhrase {
            head: self.head().clone(),
            complements,
            adjuncts: self.adjuncts().iter().map(lower_vp_adjunct).collect(),
            has_object_gap,
        }
    }
}

impl<Form: TpForm, G: TpGap, A: AgreementMarker> LowerRuntime for TensePhrase<Form, G, A> {
    type Runtime = RuntimeTensePhrase;

    fn lower_runtime(&self) -> Self::Runtime {
        RuntimeTensePhrase {
            subject: self.subject_opt().map(LowerRuntime::lower_runtime),
            predicate: self.predicate().lower_runtime(),
            form: self.form(),
            negative: self.is_negative(),
            agreement: A::agreement()
                .or_else(|| self.subject_opt().map(agreement_from_dp))
                .or_else(G::subject_agreement),
        }
    }
}

impl LowerRuntime for ContentClause {
    type Runtime = RuntimeContentClause;

    fn lower_runtime(&self) -> Self::Runtime {
        RuntimeContentClause {
            head: self.head(),
            complement: Box::new(self.complement().lower_runtime()),
        }
    }
}

impl<G: RelativeTpGap> LowerRuntime for RelativeClause<G>
where
    crate::syntax::RelativeForce: CpForce<G, Head = Relativizer>,
{
    type Runtime = RuntimeRelativeClause;

    fn lower_runtime(&self) -> Self::Runtime {
        RuntimeRelativeClause {
            head: self.head(),
            complement: Box::new(self.complement().lower_runtime()),
        }
    }
}
