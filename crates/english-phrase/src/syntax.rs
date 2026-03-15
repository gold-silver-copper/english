use crate::lexical::{
    AdjectiveEntry, AdverbEntry, ClauseKind, Complementizer, Countability, Definiteness,
    DeterminerEntry, Modal, NounEntry, Particle, PredicateCategory, PrepositionEntry, Pronoun,
    RelativeMarker, VerbEntry,
};
use english::{Number, Person};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Animacy {
    Animate,
    Inanimate,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Humanness {
    Human,
    NonHuman,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Case {
    Nominative,
    Accusative,
    Genitive,
    Reflexive,
    Oblique,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingKey(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyRole {
    Subject,
    DirectObject,
    IndirectObject,
    Oblique,
    Possessor,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapDependency {
    pub role: DependencyRole,
    pub binder: Option<BindingKey>,
}

impl GapDependency {
    pub fn new(role: DependencyRole) -> Self {
        Self { role, binder: None }
    }

    pub fn with_binder(mut self, binder: BindingKey) -> Self {
        self.binder = Some(binder);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgreementFeatures {
    pub person: Person,
    pub number: Number,
}

impl AgreementFeatures {
    pub fn new(person: Person, number: Number) -> Self {
        Self { person, number }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReferentialFeatures {
    pub gender: Gender,
    pub animacy: Animacy,
    pub humanness: Humanness,
    pub binding_key: Option<BindingKey>,
    pub discourse_label: Option<String>,
}

impl ReferentialFeatures {
    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.gender = gender;
        self
    }

    pub fn with_animacy(mut self, animacy: Animacy) -> Self {
        self.animacy = animacy;
        self
    }

    pub fn with_humanness(mut self, humanness: Humanness) -> Self {
        self.humanness = humanness;
        self
    }

    pub fn with_binding_key(mut self, key: BindingKey) -> Self {
        self.binding_key = Some(key);
        self
    }

    pub fn with_discourse_label(mut self, label: impl Into<String>) -> Self {
        self.discourse_label = Some(label.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MorphosyntacticFeatures {
    pub case: Case,
    pub definiteness: Definiteness,
    pub countability: Countability,
}

impl Default for MorphosyntacticFeatures {
    fn default() -> Self {
        Self {
            case: Case::Unspecified,
            definiteness: Definiteness::Unknown,
            countability: Countability::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DpSemantics {
    pub agreement: AgreementFeatures,
    pub reference: ReferentialFeatures,
    pub morphosyntax: MorphosyntacticFeatures,
}

impl DpSemantics {
    pub fn new(person: Person, number: Number) -> Self {
        Self {
            agreement: AgreementFeatures::new(person, number),
            reference: ReferentialFeatures::default(),
            morphosyntax: MorphosyntacticFeatures::default(),
        }
    }

    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.reference.gender = gender;
        self
    }

    pub fn with_animacy(mut self, animacy: Animacy) -> Self {
        self.reference.animacy = animacy;
        self
    }

    pub fn with_humanness(mut self, humanness: Humanness) -> Self {
        self.reference.humanness = humanness;
        self
    }

    pub fn with_case(mut self, case: Case) -> Self {
        self.morphosyntax.case = case;
        self
    }

    pub fn with_definiteness(mut self, definiteness: Definiteness) -> Self {
        self.morphosyntax.definiteness = definiteness;
        self
    }

    pub fn with_countability(mut self, countability: Countability) -> Self {
        self.morphosyntax.countability = countability;
        self
    }

    pub fn with_binding_key(mut self, key: BindingKey) -> Self {
        self.reference.binding_key = Some(key);
        self
    }

    pub fn agreement_tuple(&self) -> (Person, Number) {
        (self.agreement.person.clone(), self.agreement.number.clone())
    }

    pub fn reflexive_form(&self) -> &'static str {
        match (
            self.agreement.person.clone(),
            self.agreement.number.clone(),
            self.reference.gender,
            self.reference.animacy,
        ) {
            (Person::First, Number::Singular, _, _) => "myself",
            (Person::Second, Number::Singular, _, _) => "yourself",
            (Person::Third, Number::Singular, Gender::Masculine, _) => "himself",
            (Person::Third, Number::Singular, Gender::Feminine, _) => "herself",
            (Person::Third, Number::Singular, Gender::Neuter, _) => "itself",
            (Person::Third, Number::Singular, _, Animacy::Inanimate) => "itself",
            (Person::Third, Number::Singular, _, Animacy::Animate) => "themself",
            (Person::Third, Number::Singular, _, Animacy::Unknown) => "itself",
            (Person::First, Number::Plural, _, _) => "ourselves",
            (Person::Second, Number::Plural, _, _) => "yourselves",
            (Person::Third, Number::Plural, _, _) => "themselves",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Quantity {
    #[default]
    Singular,
    Plural,
    Count(u32),
}

impl Quantity {
    pub fn number(self) -> Number {
        match self {
            Quantity::Singular => Number::Singular,
            Quantity::Plural => Number::Plural,
            Quantity::Count(1) => Number::Singular,
            Quantity::Count(_) => Number::Plural,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Degree {
    #[default]
    Positive,
    Comparative,
    Superlative,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdverbPhrase {
    pub specifier: Option<Box<AdverbPhrase>>,
    pub head: AdverbEntry,
    pub complements: Vec<PrepositionalPhrase>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrepositionalComplement {
    DeterminerPhrase(Box<DeterminerPhrase>),
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepositionalPhrase {
    pub head: PrepositionEntry,
    pub complement: PrepositionalComplement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdjectiveComplement {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjectivePhrase {
    pub specifier: Option<Box<AdverbPhrase>>,
    pub head: AdjectiveEntry,
    pub degree: Degree,
    pub complements: Vec<AdjectiveComplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NominalHead {
    CommonNoun(NounEntry),
    ProperName(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NominalComplement {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NominalPostmodifier {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
    RelativeClause(Box<RelativeClause>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NominalPhrase {
    pub head: NominalHead,
    pub quantity: Quantity,
    pub modifiers: Vec<AdjectivePhrase>,
    pub complements: Vec<NominalComplement>,
    pub postmodifiers: Vec<NominalPostmodifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilentDeterminerKind {
    BareNominal,
    ProperName,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeterminerHead {
    Overt(DeterminerEntry),
    Silent(SilentDeterminerKind),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionDp {
    pub specifier: Option<Box<DeterminerPhrase>>,
    pub head: DeterminerHead,
    pub nominal: Box<NominalPhrase>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeterminerPhraseKind {
    Projection(ProjectionDp),
    BarePronoun { pronoun: Pronoun },
    ReflexivePronoun { antecedent: Option<BindingKey> },
    Gap { dependency: GapDependency },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeterminerPhrase {
    pub kind: DeterminerPhraseKind,
    pub semantics: DpSemantics,
}

impl DeterminerPhrase {
    pub fn semantics(&self) -> &DpSemantics {
        &self.semantics
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PredicateComplement {
    AdjectivePhrase(Box<AdjectivePhrase>),
    DeterminerPhrase(Box<DeterminerPhrase>),
}

impl PredicateComplement {
    pub fn category(&self) -> PredicateCategory {
        match self {
            PredicateComplement::AdjectivePhrase(_) => PredicateCategory::Adjective,
            PredicateComplement::DeterminerPhrase(_) => PredicateCategory::DeterminerPhrase,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClausalComplement {
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
}

impl ClausalComplement {
    pub fn clause_kind(&self) -> ClauseKind {
        match self {
            ClausalComplement::ComplementizerPhrase(_) => ClauseKind::Finite,
            ClausalComplement::NonFiniteClause(clause) => match clause.finiteness {
                Finiteness::Finite => ClauseKind::Finite,
                Finiteness::BareInfinitive => ClauseKind::BareInfinitive,
                Finiteness::ToInfinitive => ClauseKind::ToInfinitive,
                Finiteness::GerundParticiple => ClauseKind::GerundParticiple,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerbAdjunct {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    AdverbPhrase(Box<AdverbPhrase>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObliqueArgument {
    pub role: Option<String>,
    pub phrase: PrepositionalPhrase,
}

impl ObliqueArgument {
    pub fn new(phrase: PrepositionalPhrase) -> Self {
        Self { role: None, phrase }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArgumentStructure {
    pub direct_object: Option<Box<DeterminerPhrase>>,
    pub indirect_object: Option<Box<DeterminerPhrase>>,
    pub obliques: Vec<ObliqueArgument>,
    pub adjuncts: Vec<VerbAdjunct>,
    pub predicative_complement: Option<PredicateComplement>,
    pub clausal_complement: Option<ClausalComplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase {
    pub head: VerbEntry,
    pub particle: Option<Particle>,
    pub arguments: ArgumentStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Voice {
    Active,
    Passive,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoicePhrase {
    pub head: Voice,
    pub complement: Box<VerbPhrase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressivePhrase {
    pub complement: Box<VerbalProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerfectPhrase {
    pub complement: Box<VerbalProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NegativePhrase {
    pub complement: Box<VerbalProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModalPhrase {
    pub head: Modal,
    pub complement: Box<VerbalProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerbalProjection {
    Modal(ModalPhrase),
    Negative(NegativePhrase),
    Perfect(PerfectPhrase),
    Progressive(ProgressivePhrase),
    Voice(VoicePhrase),
    Verb(VerbPhrase),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tense {
    Present,
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finiteness {
    Finite,
    BareInfinitive,
    ToInfinitive,
    GerundParticiple,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhrase {
    pub subject: DeterminerPhrase,
    pub tense: Tense,
    pub predicate: VerbalProjection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonFiniteClause {
    pub finiteness: Finiteness,
    pub predicate: VerbalProjection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplementizerPhrase {
    pub head: Complementizer,
    pub complement: Box<TensePhrase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelativeClause {
    pub marker: RelativeMarker,
    pub clause: Box<TensePhrase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    pub tense_phrase: TensePhrase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminal {
    Period,
    QuestionMark,
    ExclamationMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sentence {
    pub clause: Clause,
    pub capitalize: bool,
    pub terminal: Option<Terminal>,
}
