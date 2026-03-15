use crate::derivation::{
    Diagnostic, DiagnosticBag, FiniteDerivationSpec, NonFiniteDerivationSpec,
    derive_non_finite_clause, derive_tense_phrase,
};
use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Complementizer, Countability, Definiteness, Determiner,
    DeterminerEntry, LexicalAnimacy, Modal, NounEntry, Particle, PrepositionEntry, Pronoun,
    RelativeMarker, VerbEntry,
};
use crate::syntax::{
    AdjectiveComplement, AdjectivePhrase, AdverbPhrase, Animacy, ArgumentStructure, BindingKey,
    Case, ClausalComplement, Clause, ComplementizerPhrase, DeterminerHead, DeterminerPhrase,
    DeterminerPhraseKind, DpSemantics, Finiteness, GapDependency, Gender, Humanness,
    MorphosyntacticFeatures, NominalComplement, NominalHead, NominalPhrase, NominalPostmodifier,
    NonFiniteClause, ObliqueArgument, PredicateComplement, PrepositionalComplement,
    PrepositionalPhrase, ProjectionDp, Quantity, ReferentialFeatures, RelativeClause, Sentence,
    SilentDeterminerKind, Tense, TensePhrase, Terminal, VerbAdjunct, VerbPhrase,
};
use english::{Number, Person};

fn animacy_from_lexical(animacy: LexicalAnimacy) -> Animacy {
    match animacy {
        LexicalAnimacy::Animate => Animacy::Animate,
        LexicalAnimacy::Inanimate => Animacy::Inanimate,
        LexicalAnimacy::Unknown => Animacy::Unknown,
    }
}

fn semantics_from_nominal(head: &NominalHead, quantity: Quantity) -> DpSemantics {
    match head {
        NominalHead::CommonNoun(noun) => DpSemantics {
            agreement: crate::syntax::AgreementFeatures::new(Person::Third, quantity.number()),
            reference: ReferentialFeatures::default()
                .with_animacy(animacy_from_lexical(noun.default_animacy())),
            morphosyntax: MorphosyntacticFeatures {
                case: Case::Unspecified,
                definiteness: Definiteness::Bare,
                countability: noun.countability(),
            },
        },
        NominalHead::ProperName(_) => DpSemantics {
            agreement: crate::syntax::AgreementFeatures::new(Person::Third, Number::Singular),
            reference: ReferentialFeatures::default()
                .with_animacy(Animacy::Animate)
                .with_humanness(Humanness::Human),
            morphosyntax: MorphosyntacticFeatures {
                case: Case::Unspecified,
                definiteness: Definiteness::ProperName,
                countability: Countability::ProperName,
            },
        },
    }
}

fn semantics_from_pronoun(pronoun: Pronoun) -> DpSemantics {
    match pronoun {
        Pronoun::I => DpSemantics::new(Person::First, Number::Singular)
            .with_animacy(Animacy::Animate)
            .with_humanness(Humanness::Human)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
        Pronoun::YouSingular => DpSemantics::new(Person::Second, Number::Singular)
            .with_animacy(Animacy::Animate)
            .with_humanness(Humanness::Human)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
        Pronoun::YouPlural => DpSemantics::new(Person::Second, Number::Plural)
            .with_animacy(Animacy::Animate)
            .with_humanness(Humanness::Human)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
        Pronoun::He => DpSemantics::new(Person::Third, Number::Singular)
            .with_gender(Gender::Masculine)
            .with_animacy(Animacy::Animate)
            .with_humanness(Humanness::Human)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
        Pronoun::She => DpSemantics::new(Person::Third, Number::Singular)
            .with_gender(Gender::Feminine)
            .with_animacy(Animacy::Animate)
            .with_humanness(Humanness::Human)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
        Pronoun::It => DpSemantics::new(Person::Third, Number::Singular)
            .with_gender(Gender::Neuter)
            .with_animacy(Animacy::Inanimate)
            .with_humanness(Humanness::NonHuman)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
        Pronoun::We => DpSemantics::new(Person::First, Number::Plural)
            .with_animacy(Animacy::Animate)
            .with_humanness(Humanness::Human)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
        Pronoun::They => DpSemantics::new(Person::Third, Number::Plural)
            .with_animacy(Animacy::Unknown)
            .with_definiteness(Definiteness::PronounLike)
            .with_countability(Countability::PronounLike),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdverbPhraseBuilder {
    specifier: Option<AdverbPhrase>,
    head: AdverbEntry,
    complements: Vec<PrepositionalPhrase>,
}

impl AdverbPhraseBuilder {
    pub fn new(head: impl Into<AdverbEntry>) -> Self {
        Self {
            specifier: None,
            head: head.into(),
            complements: Vec::new(),
        }
    }

    pub fn specifier(mut self, specifier: AdverbPhrase) -> Self {
        self.specifier = Some(specifier);
        self
    }

    pub fn complement(mut self, complement: PrepositionalPhrase) -> Self {
        self.complements.push(complement);
        self
    }

    pub fn build(self) -> AdverbPhrase {
        AdverbPhrase {
            specifier: self.specifier.map(Box::new),
            head: self.head,
            complements: self.complements,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepositionalPhraseBuilder {
    head: PrepositionEntry,
    complement: PrepositionalComplement,
}

impl PrepositionalPhraseBuilder {
    pub fn new(
        head: impl Into<PrepositionEntry>,
        complement: impl Into<PrepositionalComplement>,
    ) -> Self {
        Self {
            head: head.into(),
            complement: complement.into(),
        }
    }

    pub fn build(self) -> PrepositionalPhrase {
        PrepositionalPhrase {
            head: self.head,
            complement: self.complement,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjectivePhraseBuilder {
    specifier: Option<AdverbPhrase>,
    head: AdjectiveEntry,
    degree: crate::syntax::Degree,
    complements: Vec<AdjectiveComplement>,
}

impl AdjectivePhraseBuilder {
    pub fn new(head: impl Into<AdjectiveEntry>) -> Self {
        Self {
            specifier: None,
            head: head.into(),
            degree: crate::syntax::Degree::Positive,
            complements: Vec::new(),
        }
    }

    pub fn positive(mut self) -> Self {
        self.degree = crate::syntax::Degree::Positive;
        self
    }

    pub fn comparative(mut self) -> Self {
        self.degree = crate::syntax::Degree::Comparative;
        self
    }

    pub fn superlative(mut self) -> Self {
        self.degree = crate::syntax::Degree::Superlative;
        self
    }

    pub fn intensifier(mut self, specifier: AdverbPhrase) -> Self {
        self.specifier = Some(specifier);
        self
    }

    pub fn complement(mut self, complement: impl Into<AdjectiveComplement>) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn build(self) -> AdjectivePhrase {
        AdjectivePhrase {
            specifier: self.specifier.map(Box::new),
            head: self.head,
            degree: self.degree,
            complements: self.complements,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NominalPhraseBuilder {
    head: NominalHead,
    quantity: Quantity,
    modifiers: Vec<AdjectivePhrase>,
    complements: Vec<NominalComplement>,
    postmodifiers: Vec<NominalPostmodifier>,
}

impl NominalPhraseBuilder {
    pub fn common_noun(head: impl Into<NounEntry>) -> Self {
        Self {
            head: NominalHead::CommonNoun(head.into()),
            quantity: Quantity::Singular,
            modifiers: Vec::new(),
            complements: Vec::new(),
            postmodifiers: Vec::new(),
        }
    }

    pub fn proper_name(name: impl Into<String>) -> Self {
        Self {
            head: NominalHead::ProperName(name.into()),
            quantity: Quantity::Singular,
            modifiers: Vec::new(),
            complements: Vec::new(),
            postmodifiers: Vec::new(),
        }
    }

    pub fn singular(mut self) -> Self {
        self.quantity = Quantity::Singular;
        self
    }

    pub fn plural(mut self) -> Self {
        self.quantity = Quantity::Plural;
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.quantity = Quantity::Count(count);
        self
    }

    pub fn modifier(mut self, modifier: AdjectivePhrase) -> Self {
        self.modifiers.push(modifier);
        self
    }

    pub fn complement(mut self, complement: PrepositionalPhrase) -> Self {
        self.complements
            .push(NominalComplement::PrepositionalPhrase(Box::new(complement)));
        self
    }

    pub fn postmodifier(mut self, postmodifier: impl Into<NominalPostmodifier>) -> Self {
        self.postmodifiers.push(postmodifier.into());
        self
    }

    pub fn build(self) -> NominalPhrase {
        NominalPhrase {
            head: self.head,
            quantity: self.quantity,
            modifiers: self.modifiers,
            complements: self.complements,
            postmodifiers: self.postmodifiers,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum DpBuilderState {
    Projection {
        specifier: Option<DeterminerPhrase>,
        determiner: Option<DeterminerEntry>,
        nominal: NominalPhrase,
        silent: SilentDeterminerKind,
    },
    BarePronoun {
        pronoun: Pronoun,
    },
    Reflexive {
        antecedent: Option<BindingKey>,
        semantics: DpSemantics,
    },
    Gap {
        dependency: GapDependency,
        semantics: DpSemantics,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeterminerPhraseBuilder {
    state: DpBuilderState,
    semantics: DpSemantics,
    diagnostics: DiagnosticBag,
}

impl DeterminerPhraseBuilder {
    pub fn common_noun(head: impl Into<NounEntry>) -> Self {
        let nominal = NominalPhraseBuilder::common_noun(head).build();
        let semantics = semantics_from_nominal(&nominal.head, nominal.quantity);
        Self {
            state: DpBuilderState::Projection {
                specifier: None,
                determiner: None,
                nominal,
                silent: SilentDeterminerKind::BareNominal,
            },
            semantics,
            diagnostics: DiagnosticBag::default(),
        }
    }

    pub fn from_nominal(nominal: NominalPhrase) -> Self {
        let semantics = semantics_from_nominal(&nominal.head, nominal.quantity);
        let silent = match nominal.head {
            NominalHead::ProperName(_) => SilentDeterminerKind::ProperName,
            NominalHead::CommonNoun(_) => SilentDeterminerKind::BareNominal,
        };
        Self {
            state: DpBuilderState::Projection {
                specifier: None,
                determiner: None,
                nominal,
                silent,
            },
            semantics,
            diagnostics: DiagnosticBag::default(),
        }
    }

    pub fn proper_name(name: impl Into<String>) -> Self {
        Self::from_nominal(NominalPhraseBuilder::proper_name(name).build())
    }

    pub fn pronoun(pronoun: Pronoun) -> Self {
        Self {
            state: DpBuilderState::BarePronoun { pronoun },
            semantics: semantics_from_pronoun(pronoun).with_case(Case::Nominative),
            diagnostics: DiagnosticBag::default(),
        }
    }

    pub fn reflexive_from(antecedent: &DeterminerPhrase) -> Self {
        Self {
            state: DpBuilderState::Reflexive {
                antecedent: antecedent.semantics.reference.binding_key,
                semantics: antecedent
                    .semantics
                    .clone()
                    .with_case(Case::Reflexive)
                    .with_definiteness(Definiteness::PronounLike)
                    .with_countability(Countability::PronounLike),
            },
            semantics: antecedent
                .semantics
                .clone()
                .with_case(Case::Reflexive)
                .with_definiteness(Definiteness::PronounLike)
                .with_countability(Countability::PronounLike),
            diagnostics: DiagnosticBag::default(),
        }
    }

    pub fn gap(dependency: GapDependency, semantics: DpSemantics) -> Self {
        Self {
            state: DpBuilderState::Gap {
                dependency,
                semantics: semantics.clone(),
            },
            semantics,
            diagnostics: DiagnosticBag::default(),
        }
    }

    pub fn possessor(mut self, possessor: DeterminerPhrase) -> Self {
        match &mut self.state {
            DpBuilderState::Projection { specifier, .. } => *specifier = Some(possessor),
            _ => self.diagnostics.push(Diagnostic::new(
                "invalid-possessor",
                "only projected DPs can host possessors",
            )),
        }
        self
    }

    pub fn determiner(mut self, determiner: Determiner) -> Self {
        match &mut self.state {
            DpBuilderState::Projection {
                determiner: head,
                silent,
                ..
            } => {
                *head = Some(DeterminerEntry::new(determiner));
                *silent = SilentDeterminerKind::BareNominal;
                self.semantics.morphosyntax.definiteness = determiner.definiteness();
            }
            _ => self.diagnostics.push(Diagnostic::new(
                "invalid-determiner",
                "only projected DPs can take an overt determiner",
            )),
        }
        self
    }

    pub fn singular(mut self) -> Self {
        if let DpBuilderState::Projection { nominal, .. } = &mut self.state {
            nominal.quantity = Quantity::Singular;
            self.semantics.agreement.number = Number::Singular;
        }
        self
    }

    pub fn plural(mut self) -> Self {
        if let DpBuilderState::Projection { nominal, .. } = &mut self.state {
            nominal.quantity = Quantity::Plural;
            self.semantics.agreement.number = Number::Plural;
        }
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        if let DpBuilderState::Projection { nominal, .. } = &mut self.state {
            nominal.quantity = Quantity::Count(count);
            self.semantics.agreement.number = if count == 1 {
                Number::Singular
            } else {
                Number::Plural
            };
        }
        self
    }

    pub fn modifier(mut self, modifier: AdjectivePhrase) -> Self {
        match &mut self.state {
            DpBuilderState::Projection { nominal, .. } => nominal.modifiers.push(modifier),
            _ => self.diagnostics.push(Diagnostic::new(
                "invalid-modifier",
                "only projected DPs can host nominal modifiers",
            )),
        }
        self
    }

    pub fn complement(mut self, complement: PrepositionalPhrase) -> Self {
        match &mut self.state {
            DpBuilderState::Projection { nominal, .. } => nominal
                .complements
                .push(NominalComplement::PrepositionalPhrase(Box::new(complement))),
            _ => self.diagnostics.push(Diagnostic::new(
                "invalid-complement",
                "only projected DPs can host nominal complements",
            )),
        }
        self
    }

    pub fn postmodifier(mut self, postmodifier: impl Into<NominalPostmodifier>) -> Self {
        match &mut self.state {
            DpBuilderState::Projection { nominal, .. } => {
                nominal.postmodifiers.push(postmodifier.into())
            }
            _ => self.diagnostics.push(Diagnostic::new(
                "invalid-postmodifier",
                "only projected DPs can host postmodifiers",
            )),
        }
        self
    }

    pub fn masculine(mut self) -> Self {
        self.semantics.reference.gender = Gender::Masculine;
        self.semantics.reference.animacy = Animacy::Animate;
        self.semantics.reference.humanness = Humanness::Human;
        self
    }

    pub fn feminine(mut self) -> Self {
        self.semantics.reference.gender = Gender::Feminine;
        self.semantics.reference.animacy = Animacy::Animate;
        self.semantics.reference.humanness = Humanness::Human;
        self
    }

    pub fn animate(mut self) -> Self {
        self.semantics.reference.animacy = Animacy::Animate;
        self
    }

    pub fn inanimate(mut self) -> Self {
        self.semantics.reference.animacy = Animacy::Inanimate;
        self
    }

    pub fn binding_key(mut self, key: BindingKey) -> Self {
        self.semantics.reference.binding_key = Some(key);
        self
    }

    pub fn semantics(mut self, semantics: DpSemantics) -> Self {
        self.semantics = semantics;
        self
    }

    pub fn build(self) -> Result<DeterminerPhrase, DiagnosticBag> {
        if !self.diagnostics.is_empty() {
            return Err(self.diagnostics);
        }

        let kind = match self.state {
            DpBuilderState::Projection {
                specifier,
                determiner,
                nominal,
                silent,
            } => DeterminerPhraseKind::Projection(ProjectionDp {
                specifier: specifier.map(Box::new),
                head: match determiner {
                    Some(head) => DeterminerHead::Overt(head),
                    None => DeterminerHead::Silent(silent),
                },
                nominal: Box::new(nominal),
            }),
            DpBuilderState::BarePronoun { pronoun } => {
                DeterminerPhraseKind::BarePronoun { pronoun }
            }
            DpBuilderState::Reflexive {
                antecedent,
                semantics,
            } => {
                return Ok(DeterminerPhrase {
                    kind: DeterminerPhraseKind::ReflexivePronoun { antecedent },
                    semantics,
                });
            }
            DpBuilderState::Gap {
                dependency,
                semantics,
            } => {
                return Ok(DeterminerPhrase {
                    kind: DeterminerPhraseKind::Gap { dependency },
                    semantics,
                });
            }
        };

        Ok(DeterminerPhrase {
            kind,
            semantics: self.semantics,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhraseBuilder {
    head: VerbEntry,
    particle: Option<Particle>,
    arguments: ArgumentStructure,
}

impl VerbPhraseBuilder {
    pub fn new(head: impl Into<VerbEntry>) -> Self {
        Self {
            head: head.into(),
            particle: None,
            arguments: ArgumentStructure::default(),
        }
    }

    pub fn particle(mut self, particle: impl Into<Particle>) -> Self {
        self.particle = Some(particle.into());
        self
    }

    pub fn direct_object(mut self, object: DeterminerPhrase) -> Self {
        self.arguments.direct_object = Some(Box::new(object));
        self
    }

    pub fn indirect_object(mut self, object: DeterminerPhrase) -> Self {
        self.arguments.indirect_object = Some(Box::new(object));
        self
    }

    pub fn oblique(mut self, phrase: PrepositionalPhrase) -> Self {
        self.arguments.obliques.push(ObliqueArgument::new(phrase));
        self
    }

    pub fn oblique_with_role(
        mut self,
        role: impl Into<String>,
        phrase: PrepositionalPhrase,
    ) -> Self {
        self.arguments
            .obliques
            .push(ObliqueArgument::new(phrase).with_role(role));
        self
    }

    pub fn adjunct(mut self, adjunct: impl Into<VerbAdjunct>) -> Self {
        self.arguments.adjuncts.push(adjunct.into());
        self
    }

    pub fn predicative_complement(mut self, complement: impl Into<PredicateComplement>) -> Self {
        self.arguments.predicative_complement = Some(complement.into());
        self
    }

    pub fn clausal_complement(mut self, complement: impl Into<ClausalComplement>) -> Self {
        self.arguments.clausal_complement = Some(complement.into());
        self
    }

    pub fn build(self) -> VerbPhrase {
        VerbPhrase {
            head: self.head,
            particle: self.particle,
            arguments: self.arguments,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhraseBuilder {
    subject: DeterminerPhrase,
    predicate: VerbPhrase,
    derivation: FiniteDerivationSpec,
}

impl TensePhraseBuilder {
    pub fn new(subject: DeterminerPhrase, predicate: VerbPhrase) -> Self {
        Self {
            subject,
            predicate,
            derivation: FiniteDerivationSpec::default(),
        }
    }

    pub fn present(mut self) -> Self {
        self.derivation.tense = Tense::Present;
        self
    }

    pub fn past(mut self) -> Self {
        self.derivation.tense = Tense::Past;
        self
    }

    pub fn modal(mut self, modal: Modal) -> Self {
        self.derivation.modal = Some(modal);
        self
    }

    pub fn negative(mut self) -> Self {
        self.derivation.negative = true;
        self
    }

    pub fn perfect(mut self) -> Self {
        self.derivation.perfect = true;
        self
    }

    pub fn progressive(mut self) -> Self {
        self.derivation.progressive = true;
        self
    }

    pub fn passive(mut self) -> Self {
        self.derivation.passive = true;
        self
    }

    pub fn reflexive(mut self) -> Self {
        self.derivation.reflexive = true;
        self
    }

    pub fn causative(mut self, causer: DeterminerPhrase) -> Self {
        self.derivation.causative_causer = Some(causer);
        self
    }

    pub fn build(self) -> Result<TensePhrase, DiagnosticBag> {
        derive_tense_phrase(self.subject, self.predicate, self.derivation)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonFiniteClauseBuilder {
    predicate: VerbPhrase,
    derivation: NonFiniteDerivationSpec,
}

impl NonFiniteClauseBuilder {
    pub fn new(predicate: VerbPhrase) -> Self {
        Self {
            predicate,
            derivation: NonFiniteDerivationSpec::default(),
        }
    }

    pub fn bare_infinitive(mut self) -> Self {
        self.derivation.finiteness = Finiteness::BareInfinitive;
        self
    }

    pub fn to_infinitive(mut self) -> Self {
        self.derivation.finiteness = Finiteness::ToInfinitive;
        self
    }

    pub fn gerund_participle(mut self) -> Self {
        self.derivation.finiteness = Finiteness::GerundParticiple;
        self
    }

    pub fn negative(mut self) -> Self {
        self.derivation.negative = true;
        self
    }

    pub fn perfect(mut self) -> Self {
        self.derivation.perfect = true;
        self
    }

    pub fn progressive(mut self) -> Self {
        self.derivation.progressive = true;
        self
    }

    pub fn passive(mut self) -> Self {
        self.derivation.passive = true;
        self
    }

    pub fn build(self) -> Result<NonFiniteClause, DiagnosticBag> {
        derive_non_finite_clause(self.predicate, self.derivation)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplementizerPhraseBuilder {
    head: Complementizer,
    complement: TensePhrase,
}

impl ComplementizerPhraseBuilder {
    pub fn new(head: Complementizer, complement: TensePhrase) -> Self {
        Self { head, complement }
    }

    pub fn that(complement: TensePhrase) -> Self {
        Self::new(Complementizer::that(), complement)
    }

    pub fn null(complement: TensePhrase) -> Self {
        Self::new(Complementizer::null(), complement)
    }

    pub fn relative(marker: RelativeMarker, complement: TensePhrase) -> Self {
        Self::new(Complementizer::relative(marker), complement)
    }

    pub fn build(self) -> ComplementizerPhrase {
        ComplementizerPhrase {
            head: self.head,
            complement: Box::new(self.complement),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SentenceBuilder {
    clause: Clause,
    capitalize: bool,
    terminal: Option<Terminal>,
}

impl SentenceBuilder {
    pub fn new(clause: Clause) -> Self {
        Self {
            clause,
            capitalize: false,
            terminal: None,
        }
    }

    pub fn capitalize(mut self) -> Self {
        self.capitalize = true;
        self
    }

    pub fn period(mut self) -> Self {
        self.terminal = Some(Terminal::Period);
        self
    }

    pub fn question_mark(mut self) -> Self {
        self.terminal = Some(Terminal::QuestionMark);
        self
    }

    pub fn exclamation_mark(mut self) -> Self {
        self.terminal = Some(Terminal::ExclamationMark);
        self
    }

    pub fn build(self) -> Sentence {
        Sentence {
            clause: self.clause,
            capitalize: self.capitalize,
            terminal: self.terminal,
        }
    }
}

impl From<DeterminerPhrase> for PrepositionalComplement {
    fn from(value: DeterminerPhrase) -> Self {
        PrepositionalComplement::DeterminerPhrase(Box::new(value))
    }
}

impl From<ComplementizerPhrase> for PrepositionalComplement {
    fn from(value: ComplementizerPhrase) -> Self {
        PrepositionalComplement::ComplementizerPhrase(Box::new(value))
    }
}

impl From<NonFiniteClause> for PrepositionalComplement {
    fn from(value: NonFiniteClause) -> Self {
        PrepositionalComplement::NonFiniteClause(Box::new(value))
    }
}

impl From<PrepositionalPhrase> for AdjectiveComplement {
    fn from(value: PrepositionalPhrase) -> Self {
        AdjectiveComplement::PrepositionalPhrase(Box::new(value))
    }
}

impl From<ComplementizerPhrase> for AdjectiveComplement {
    fn from(value: ComplementizerPhrase) -> Self {
        AdjectiveComplement::ComplementizerPhrase(Box::new(value))
    }
}

impl From<NonFiniteClause> for AdjectiveComplement {
    fn from(value: NonFiniteClause) -> Self {
        AdjectiveComplement::NonFiniteClause(Box::new(value))
    }
}

impl From<PrepositionalPhrase> for NominalPostmodifier {
    fn from(value: PrepositionalPhrase) -> Self {
        NominalPostmodifier::PrepositionalPhrase(Box::new(value))
    }
}

impl From<ComplementizerPhrase> for NominalPostmodifier {
    fn from(value: ComplementizerPhrase) -> Self {
        NominalPostmodifier::ComplementizerPhrase(Box::new(value))
    }
}

impl From<NonFiniteClause> for NominalPostmodifier {
    fn from(value: NonFiniteClause) -> Self {
        NominalPostmodifier::NonFiniteClause(Box::new(value))
    }
}

impl From<RelativeClause> for NominalPostmodifier {
    fn from(value: RelativeClause) -> Self {
        NominalPostmodifier::RelativeClause(Box::new(value))
    }
}

impl From<AdjectivePhrase> for PredicateComplement {
    fn from(value: AdjectivePhrase) -> Self {
        PredicateComplement::AdjectivePhrase(Box::new(value))
    }
}

impl From<DeterminerPhrase> for PredicateComplement {
    fn from(value: DeterminerPhrase) -> Self {
        PredicateComplement::DeterminerPhrase(Box::new(value))
    }
}

impl From<ComplementizerPhrase> for ClausalComplement {
    fn from(value: ComplementizerPhrase) -> Self {
        ClausalComplement::ComplementizerPhrase(Box::new(value))
    }
}

impl From<NonFiniteClause> for ClausalComplement {
    fn from(value: NonFiniteClause) -> Self {
        ClausalComplement::NonFiniteClause(Box::new(value))
    }
}

impl From<PrepositionalPhrase> for VerbAdjunct {
    fn from(value: PrepositionalPhrase) -> Self {
        VerbAdjunct::PrepositionalPhrase(Box::new(value))
    }
}

impl From<AdverbPhrase> for VerbAdjunct {
    fn from(value: AdverbPhrase) -> Self {
        VerbAdjunct::AdverbPhrase(Box::new(value))
    }
}
