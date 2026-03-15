use english_core::EnglishCore;

pub use english::{Adj, Noun, Number, Person, Verb};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adverb(String);

impl Adverb {
    pub fn new(head: impl Into<String>) -> Self {
        Self(head.into())
    }

    fn render(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Adverb {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for Adverb {
    fn from(text: String) -> Self {
        Self(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preposition(String);

impl Preposition {
    pub fn new(head: impl Into<String>) -> Self {
        Self(head.into())
    }

    fn render(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Preposition {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for Preposition {
    fn from(text: String) -> Self {
        Self(text)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdverbPhrase {
    specifier: Option<Box<AdverbPhrase>>,
    head: Adverb,
    complements: Vec<PrepositionalPhrase>,
}

impl AdverbPhrase {
    pub fn new(head: impl Into<Adverb>) -> Self {
        Self {
            head: head.into(),
            specifier: None,
            complements: Vec::new(),
        }
    }

    pub fn specifier(mut self, specifier: AdverbPhrase) -> Self {
        self.specifier = Some(Box::new(specifier));
        self
    }

    pub fn complement(mut self, complement: PrepositionalPhrase) -> Self {
        self.complements.push(complement);
        self
    }

    pub fn render(&self) -> String {
        let mut parts = Vec::new();
        if let Some(specifier) = &self.specifier {
            parts.push(specifier.render());
        }
        parts.push(self.head.render().to_string());
        parts.extend(self.complements.iter().map(PrepositionalPhrase::render));
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepositionalPhrase {
    head: Preposition,
    complement: DeterminerPhrase,
}

impl PrepositionalPhrase {
    pub fn new(preposition: impl Into<Preposition>, complement: DeterminerPhrase) -> Self {
        Self {
            head: preposition.into(),
            complement,
        }
    }

    pub fn render(&self) -> String {
        format!("{} {}", self.head.render(), self.complement.render())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Determiner {
    The,
    A,
    An,
    This,
    That,
    These,
    Those,
    Custom(String),
}

impl Determiner {
    pub fn the() -> Self {
        Self::The
    }

    pub fn a() -> Self {
        Self::A
    }

    pub fn an() -> Self {
        Self::An
    }

    pub fn this() -> Self {
        Self::This
    }

    pub fn that() -> Self {
        Self::That
    }

    pub fn these() -> Self {
        Self::These
    }

    pub fn those() -> Self {
        Self::Those
    }

    pub fn custom(text: impl Into<String>) -> Self {
        Self::Custom(text.into())
    }

    fn render(&self) -> &str {
        match self {
            Determiner::The => "the",
            Determiner::A => "a",
            Determiner::An => "an",
            Determiner::This => "this",
            Determiner::That => "that",
            Determiner::These => "these",
            Determiner::Those => "those",
            Determiner::Custom(text) => text.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pronoun {
    I,
    YouSingular,
    YouPlural,
    He,
    She,
    It,
    We,
    They,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct DpSemantics {
    pub person: Person,
    pub number: Number,
    pub gender: Gender,
    pub animacy: Animacy,
}

impl DpSemantics {
    pub fn new(person: Person, number: Number) -> Self {
        Self {
            person,
            number,
            gender: Gender::Unknown,
            animacy: Animacy::Unknown,
        }
    }

    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.gender = gender;
        self
    }

    pub fn with_animacy(mut self, animacy: Animacy) -> Self {
        self.animacy = animacy;
        self
    }

    pub fn agreement(&self) -> (Person, Number) {
        (self.person.clone(), self.number.clone())
    }

    pub fn reflexive_form(&self) -> &'static str {
        match (
            self.person.clone(),
            self.number.clone(),
            self.gender,
            self.animacy,
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

impl Pronoun {
    pub fn i() -> Self {
        Self::I
    }

    pub fn you_singular() -> Self {
        Self::YouSingular
    }

    pub fn you_plural() -> Self {
        Self::YouPlural
    }

    pub fn he() -> Self {
        Self::He
    }

    pub fn she() -> Self {
        Self::She
    }

    pub fn it() -> Self {
        Self::It
    }

    pub fn we() -> Self {
        Self::We
    }

    pub fn they() -> Self {
        Self::They
    }

    fn render(self) -> &'static str {
        match self {
            Pronoun::I => "I",
            Pronoun::YouSingular | Pronoun::YouPlural => "you",
            Pronoun::He => "he",
            Pronoun::She => "she",
            Pronoun::It => "it",
            Pronoun::We => "we",
            Pronoun::They => "they",
        }
    }

    fn possessive_determiner(self) -> &'static str {
        match self {
            Pronoun::I => "my",
            Pronoun::YouSingular | Pronoun::YouPlural => "your",
            Pronoun::He => "his",
            Pronoun::She => "her",
            Pronoun::It => "its",
            Pronoun::We => "our",
            Pronoun::They => "their",
        }
    }

    fn semantics(self) -> DpSemantics {
        match self {
            Pronoun::I => {
                DpSemantics::new(Person::First, Number::Singular).with_animacy(Animacy::Animate)
            }
            Pronoun::YouSingular => {
                DpSemantics::new(Person::Second, Number::Singular).with_animacy(Animacy::Animate)
            }
            Pronoun::YouPlural => {
                DpSemantics::new(Person::Second, Number::Plural).with_animacy(Animacy::Animate)
            }
            Pronoun::He => DpSemantics::new(Person::Third, Number::Singular)
                .with_gender(Gender::Masculine)
                .with_animacy(Animacy::Animate),
            Pronoun::She => DpSemantics::new(Person::Third, Number::Singular)
                .with_gender(Gender::Feminine)
                .with_animacy(Animacy::Animate),
            Pronoun::It => DpSemantics::new(Person::Third, Number::Singular)
                .with_gender(Gender::Neuter)
                .with_animacy(Animacy::Inanimate),
            Pronoun::We => {
                DpSemantics::new(Person::First, Number::Plural).with_animacy(Animacy::Animate)
            }
            Pronoun::They => {
                DpSemantics::new(Person::Third, Number::Plural).with_animacy(Animacy::Unknown)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeMarker {
    Who,
    Which,
    That,
    Bare,
    Custom(String),
}

impl RelativeMarker {
    fn render(&self) -> &str {
        match self {
            RelativeMarker::Who => "who",
            RelativeMarker::Which => "which",
            RelativeMarker::That => "that",
            RelativeMarker::Bare => "",
            RelativeMarker::Custom(text) => text.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complementizer {
    That,
    Null,
    Relative(RelativeMarker),
    Custom(String),
}

impl Complementizer {
    pub fn that() -> Self {
        Self::That
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn relative(marker: RelativeMarker) -> Self {
        Self::Relative(marker)
    }

    pub fn custom(text: impl Into<String>) -> Self {
        Self::Custom(text.into())
    }

    fn render(&self) -> &str {
        match self {
            Complementizer::That => "that",
            Complementizer::Null => "",
            Complementizer::Relative(marker) => marker.render(),
            Complementizer::Custom(text) => text.as_str(),
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
pub enum AdjComplement {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
}

impl AdjComplement {
    fn render(&self) -> String {
        match self {
            AdjComplement::PrepositionalPhrase(phrase) => phrase.render(),
            AdjComplement::ComplementizerPhrase(phrase) => phrase.render(),
            AdjComplement::NonFiniteClause(clause) => clause.render(),
        }
    }
}

impl From<PrepositionalPhrase> for AdjComplement {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

impl From<ComplementizerPhrase> for AdjComplement {
    fn from(phrase: ComplementizerPhrase) -> Self {
        Self::ComplementizerPhrase(Box::new(phrase))
    }
}

impl From<NonFiniteClause> for AdjComplement {
    fn from(clause: NonFiniteClause) -> Self {
        Self::NonFiniteClause(Box::new(clause))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjPhrase {
    head: Adj,
    degree: Degree,
    intensifier: Option<Box<AdverbPhrase>>,
    complements: Vec<AdjComplement>,
}

impl AdjPhrase {
    pub fn new<T: Into<Adj>>(head: T) -> Self {
        Self {
            head: head.into(),
            degree: Degree::Positive,
            intensifier: None,
            complements: Vec::new(),
        }
    }

    pub fn positive(mut self) -> Self {
        self.degree = Degree::Positive;
        self
    }

    pub fn comparative(mut self) -> Self {
        self.degree = Degree::Comparative;
        self
    }

    pub fn superlative(mut self) -> Self {
        self.degree = Degree::Superlative;
        self
    }

    pub fn intensifier(mut self, intensifier: AdverbPhrase) -> Self {
        self.intensifier = Some(Box::new(intensifier));
        self
    }

    pub fn complement<C: Into<AdjComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        if let Some(intensifier) = &self.intensifier {
            parts.push(intensifier.render());
        }

        parts.push(match self.degree {
            Degree::Positive => self.head.positive(),
            Degree::Comparative => self.head.comparative(),
            Degree::Superlative => self.head.superlative(),
        });

        parts.extend(self.complements.iter().map(AdjComplement::render));
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NominalComplement {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
}

impl NominalComplement {
    fn render(&self) -> String {
        match self {
            NominalComplement::PrepositionalPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<PrepositionalPhrase> for NominalComplement {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelativeClause {
    clause: Box<ComplementizerPhrase>,
}

impl RelativeClause {
    pub fn new(marker: RelativeMarker, clause: TensePhrase) -> Self {
        Self {
            clause: Box::new(ComplementizerPhrase::relative(marker, clause)),
        }
    }

    pub fn who(clause: TensePhrase) -> Self {
        Self::new(RelativeMarker::Who, clause)
    }

    pub fn which(clause: TensePhrase) -> Self {
        Self::new(RelativeMarker::Which, clause)
    }

    pub fn that(clause: TensePhrase) -> Self {
        Self::new(RelativeMarker::That, clause)
    }

    pub fn bare(clause: TensePhrase) -> Self {
        Self::new(RelativeMarker::Bare, clause)
    }

    fn render(&self) -> String {
        self.clause.render()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NominalPostmodifier {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
    RelativeClause(Box<RelativeClause>),
}

impl NominalPostmodifier {
    fn render(&self) -> String {
        match self {
            NominalPostmodifier::PrepositionalPhrase(phrase) => phrase.render(),
            NominalPostmodifier::ComplementizerPhrase(phrase) => phrase.render(),
            NominalPostmodifier::NonFiniteClause(clause) => clause.render(),
            NominalPostmodifier::RelativeClause(clause) => clause.render(),
        }
    }
}

impl From<PrepositionalPhrase> for NominalPostmodifier {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

impl From<ComplementizerPhrase> for NominalPostmodifier {
    fn from(phrase: ComplementizerPhrase) -> Self {
        Self::ComplementizerPhrase(Box::new(phrase))
    }
}

impl From<NonFiniteClause> for NominalPostmodifier {
    fn from(clause: NonFiniteClause) -> Self {
        Self::NonFiniteClause(Box::new(clause))
    }
}

impl From<RelativeClause> for NominalPostmodifier {
    fn from(clause: RelativeClause) -> Self {
        Self::RelativeClause(Box::new(clause))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Quantity {
    #[default]
    Singular,
    Plural,
    Count(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NominalPhrase {
    head: Noun,
    quantity: Quantity,
    modifiers: Vec<AdjPhrase>,
    complements: Vec<NominalComplement>,
    postmodifiers: Vec<NominalPostmodifier>,
}

impl NominalPhrase {
    pub fn new<T: Into<Noun>>(head: T) -> Self {
        Self {
            head: head.into(),
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

    pub fn modifier(mut self, modifier: AdjPhrase) -> Self {
        self.modifiers.push(modifier);
        self
    }

    pub fn complement(mut self, complement: PrepositionalPhrase) -> Self {
        self.complements.push(NominalComplement::from(complement));
        self
    }

    pub fn postmodifier<M: Into<NominalPostmodifier>>(mut self, postmodifier: M) -> Self {
        self.postmodifiers.push(postmodifier.into());
        self
    }

    pub fn agreement(&self) -> (Person, Number) {
        self.semantics().agreement()
    }

    pub fn semantics(&self) -> DpSemantics {
        let number = match self.quantity {
            Quantity::Singular => Number::Singular,
            Quantity::Plural => Number::Plural,
            Quantity::Count(1) => Number::Singular,
            Quantity::Count(_) => Number::Plural,
        };

        DpSemantics::new(Person::Third, number)
    }

    fn render_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();

        if let Quantity::Count(count) = self.quantity {
            parts.push(count.to_string());
        }

        parts.extend(self.modifiers.iter().map(AdjPhrase::render));

        let head = match self.quantity {
            Quantity::Singular => self.head.singular(),
            Quantity::Plural => self.head.plural(),
            Quantity::Count(count) => self.head.count(count),
        };
        parts.push(head);

        parts.extend(self.complements.iter().map(NominalComplement::render));
        parts.extend(self.postmodifiers.iter().map(NominalPostmodifier::render));
        parts
    }

    pub fn render(&self) -> String {
        self.render_parts().join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
enum DeterminerPhraseCore {
    Nominal(NominalPhrase),
    Pronoun(Pronoun),
    ProperName(String),
    Gap,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeterminerPhrase {
    specifier: Option<Box<DeterminerPhrase>>,
    determiner: Option<Determiner>,
    core: DeterminerPhraseCore,
    semantics: DpSemantics,
}

impl DeterminerPhrase {
    pub fn new<T: Into<Noun>>(head: T) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::Nominal(NominalPhrase::new(head)),
            semantics: DpSemantics::new(Person::Third, Number::Singular),
        }
    }

    pub fn from_nominal(nominal: NominalPhrase) -> Self {
        let semantics = nominal.semantics();
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::Nominal(nominal),
            semantics,
        }
    }

    pub fn pronoun(pronoun: Pronoun) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::Pronoun(pronoun),
            semantics: pronoun.semantics(),
        }
    }

    pub fn proper_name(name: impl Into<String>) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::ProperName(name.into()),
            semantics: DpSemantics::new(Person::Third, Number::Singular)
                .with_animacy(Animacy::Animate),
        }
    }

    pub fn gap() -> Self {
        Self::gap_with_agreement(Person::Third, Number::Singular)
    }

    pub fn gap_with_agreement(person: Person, number: Number) -> Self {
        Self::gap_with_semantics(DpSemantics::new(person, number))
    }

    pub fn gap_with_semantics(semantics: DpSemantics) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::Gap,
            semantics,
        }
    }

    pub fn specifier(mut self, specifier: DeterminerPhrase) -> Self {
        self.specifier = Some(Box::new(specifier));
        self
    }

    pub fn possessor(self, possessor: DeterminerPhrase) -> Self {
        self.specifier(possessor)
    }

    pub fn determiner(mut self, determiner: Determiner) -> Self {
        self.determiner = Some(determiner);
        self
    }

    pub fn the(self) -> Self {
        self.determiner(Determiner::the())
    }

    pub fn singular(mut self) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            *nominal = nominal.clone().singular();
        }
        self.semantics.number = Number::Singular;
        self
    }

    pub fn plural(mut self) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            *nominal = nominal.clone().plural();
        }
        self.semantics.number = Number::Plural;
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            *nominal = nominal.clone().count(count);
        }
        self.semantics.number = if count == 1 {
            Number::Singular
        } else {
            Number::Plural
        };
        self
    }

    pub fn masculine(mut self) -> Self {
        self.semantics.gender = Gender::Masculine;
        self.semantics.animacy = Animacy::Animate;
        self
    }

    pub fn feminine(mut self) -> Self {
        self.semantics.gender = Gender::Feminine;
        self.semantics.animacy = Animacy::Animate;
        self
    }

    pub fn neuter(mut self) -> Self {
        self.semantics.gender = Gender::Neuter;
        self
    }

    pub fn animate(mut self) -> Self {
        self.semantics.animacy = Animacy::Animate;
        self
    }

    pub fn inanimate(mut self) -> Self {
        self.semantics.animacy = Animacy::Inanimate;
        self
    }

    pub fn with_semantics(mut self, semantics: DpSemantics) -> Self {
        self.semantics = semantics;
        self
    }

    pub fn modifier(mut self, modifier: AdjPhrase) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            nominal.modifiers.push(modifier);
        }
        self
    }

    pub fn complement(mut self, complement: PrepositionalPhrase) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            nominal.complements.push(NominalComplement::from(complement));
        }
        self
    }

    pub fn postmodifier<M: Into<NominalPostmodifier>>(mut self, postmodifier: M) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            nominal.postmodifiers.push(postmodifier.into());
        }
        self
    }

    pub fn agreement(&self) -> (Person, Number) {
        self.semantics.agreement()
    }

    pub fn semantics(&self) -> &DpSemantics {
        &self.semantics
    }

    pub fn reflexive_form(&self) -> &'static str {
        self.semantics.reflexive_form()
    }

    fn render_possessor(&self) -> Option<String> {
        self.specifier.as_ref().map(|specifier| match &specifier.core {
            DeterminerPhraseCore::Pronoun(pronoun) => pronoun.possessive_determiner().to_string(),
            _ => {
                let text = specifier.render();
                if text.ends_with('s') {
                    format!("{text}'")
                } else {
                    format!("{text}'s")
                }
            }
        })
    }

    fn render_core(&self) -> String {
        match &self.core {
            DeterminerPhraseCore::Nominal(nominal) => nominal.render(),
            DeterminerPhraseCore::Pronoun(pronoun) => pronoun.render().to_string(),
            DeterminerPhraseCore::ProperName(name) => name.clone(),
            DeterminerPhraseCore::Gap => String::new(),
        }
    }

    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        if let Some(possessor) = self.render_possessor() {
            parts.push(possessor);
        }

        if let Some(determiner) = &self.determiner {
            parts.push(determiner.render().to_string());
        }

        parts.push(self.render_core());
        parts
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tense {
    #[default]
    Present,
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Aspect {
    #[default]
    Simple,
    Perfect,
    Progressive,
    PerfectProgressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Polarity {
    #[default]
    Affirmative,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Voice {
    #[default]
    Active,
    Passive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonFiniteForm {
    BareInfinitive,
    ToInfinitive,
    GerundParticiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    Will,
    Would,
    Can,
    Could,
    Shall,
    Should,
    May,
    Might,
    Must,
}

impl Modal {
    fn render(self) -> &'static str {
        match self {
            Modal::Will => "will",
            Modal::Would => "would",
            Modal::Can => "can",
            Modal::Could => "could",
            Modal::Shall => "shall",
            Modal::Should => "should",
            Modal::May => "may",
            Modal::Might => "might",
            Modal::Must => "must",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuxForm {
    Finite,
    Infinitive,
    PastParticiple,
    PresentParticiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainForm {
    Finite,
    Infinitive,
    PastParticiple,
    PresentParticiple,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AuxLemma {
    Modal(Modal),
    Verb(Verb),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Auxiliary {
    lemma: AuxLemma,
    form: AuxForm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerbPlan {
    auxiliaries: Vec<Auxiliary>,
    negated: bool,
    main_form: Option<MainForm>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerbComplement {
    DeterminerPhrase(Box<DeterminerPhrase>),
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    AdjectivePhrase(Box<AdjPhrase>),
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
}

impl VerbComplement {
    fn render(&self) -> String {
        match self {
            VerbComplement::DeterminerPhrase(phrase) => phrase.render(),
            VerbComplement::PrepositionalPhrase(phrase) => phrase.render(),
            VerbComplement::AdjectivePhrase(phrase) => phrase.render(),
            VerbComplement::ComplementizerPhrase(phrase) => phrase.render(),
            VerbComplement::NonFiniteClause(clause) => clause.render(),
        }
    }
}

impl From<DeterminerPhrase> for VerbComplement {
    fn from(phrase: DeterminerPhrase) -> Self {
        Self::DeterminerPhrase(Box::new(phrase))
    }
}

impl From<PrepositionalPhrase> for VerbComplement {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

impl From<AdjPhrase> for VerbComplement {
    fn from(phrase: AdjPhrase) -> Self {
        Self::AdjectivePhrase(Box::new(phrase))
    }
}

impl From<ComplementizerPhrase> for VerbComplement {
    fn from(phrase: ComplementizerPhrase) -> Self {
        Self::ComplementizerPhrase(Box::new(phrase))
    }
}

impl From<NonFiniteClause> for VerbComplement {
    fn from(clause: NonFiniteClause) -> Self {
        Self::NonFiniteClause(Box::new(clause))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerbAdjunct {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    AdverbPhrase(Box<AdverbPhrase>),
}

impl VerbAdjunct {
    fn render(&self) -> String {
        match self {
            VerbAdjunct::PrepositionalPhrase(phrase) => phrase.render(),
            VerbAdjunct::AdverbPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<PrepositionalPhrase> for VerbAdjunct {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

impl From<AdverbPhrase> for VerbAdjunct {
    fn from(phrase: AdverbPhrase) -> Self {
        Self::AdverbPhrase(Box::new(phrase))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LightVerb {
    Intransitive,
    Transitive,
    Copular,
    Clausal,
}

impl LightVerb {
    fn classify(predicate: &VerbPhrase) -> Self {
        if predicate
            .complements
            .iter()
            .any(|complement| matches!(complement, VerbComplement::AdjectivePhrase(_)))
            || predicate.head.infinitive() == "be"
        {
            Self::Copular
        } else if predicate.complements.iter().any(|complement| {
            matches!(
                complement,
                VerbComplement::ComplementizerPhrase(_) | VerbComplement::NonFiniteClause(_)
            )
        }) {
            Self::Clausal
        } else if predicate.complements.is_empty() {
            Self::Intransitive
        } else {
            Self::Transitive
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LightVerbPhrase {
    specifier: DeterminerPhrase,
    head: LightVerb,
    complement: VerbPhrase,
}

impl LightVerbPhrase {
    fn new(specifier: DeterminerPhrase, complement: VerbPhrase) -> Self {
        let head = LightVerb::classify(&complement);
        Self {
            specifier,
            head,
            complement,
        }
    }

    fn with_complement(mut self, complement: VerbPhrase) -> Self {
        self.head = LightVerb::classify(&complement);
        self.complement = complement;
        self
    }

    fn subject(&self) -> &DeterminerPhrase {
        &self.specifier
    }

    fn predicate(&self) -> &VerbPhrase {
        &self.complement
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase {
    head: Verb,
    tense: Tense,
    aspect: Aspect,
    polarity: Polarity,
    voice: Voice,
    modal: Option<Modal>,
    particle: Option<String>,
    agreement: Option<(Person, Number)>,
    complements: Vec<VerbComplement>,
    adjuncts: Vec<VerbAdjunct>,
}

impl VerbPhrase {
    pub fn new<T: Into<Verb>>(head: T) -> Self {
        Self {
            head: head.into(),
            tense: Tense::Present,
            aspect: Aspect::Simple,
            polarity: Polarity::Affirmative,
            voice: Voice::Active,
            modal: None,
            particle: None,
            agreement: None,
            complements: Vec::new(),
            adjuncts: Vec::new(),
        }
    }

    pub fn present(mut self) -> Self {
        self.tense = Tense::Present;
        self
    }

    pub fn past(mut self) -> Self {
        self.tense = Tense::Past;
        self
    }

    pub fn simple(mut self) -> Self {
        self.aspect = Aspect::Simple;
        self
    }

    pub fn perfect(mut self) -> Self {
        self.aspect = Aspect::Perfect;
        self
    }

    pub fn progressive(mut self) -> Self {
        self.aspect = Aspect::Progressive;
        self
    }

    pub fn perfect_progressive(mut self) -> Self {
        self.aspect = Aspect::PerfectProgressive;
        self
    }

    pub fn affirmative(mut self) -> Self {
        self.polarity = Polarity::Affirmative;
        self
    }

    pub fn negative(mut self) -> Self {
        self.polarity = Polarity::Negative;
        self
    }

    pub fn active(mut self) -> Self {
        self.voice = Voice::Active;
        self
    }

    pub fn passive(mut self) -> Self {
        self.voice = Voice::Passive;
        self
    }

    pub fn modal(mut self, modal: Modal) -> Self {
        self.modal = Some(modal);
        self
    }

    pub fn particle(mut self, particle: impl Into<String>) -> Self {
        self.particle = Some(particle.into());
        self
    }

    pub fn subject(mut self, person: Person, number: Number) -> Self {
        self.agreement = Some((person, number));
        self
    }

    pub fn agree_with(mut self, subject: &DeterminerPhrase) -> Self {
        self.agreement = Some(subject.agreement());
        self
    }

    pub fn complement<C: Into<VerbComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn adjunct<A: Into<VerbAdjunct>>(mut self, adjunct: A) -> Self {
        self.adjuncts.push(adjunct.into());
        self
    }

    pub fn direct_object(self, object: DeterminerPhrase) -> Self {
        self.complement(object)
    }

    pub fn pp_complement(self, phrase: PrepositionalPhrase) -> Self {
        self.complement(phrase)
    }

    pub fn predicative_complement(self, phrase: AdjPhrase) -> Self {
        self.complement(phrase)
    }

    pub fn clausal_complement(self, phrase: ComplementizerPhrase) -> Self {
        self.complement(phrase)
    }

    pub fn non_finite_complement(self, clause: NonFiniteClause) -> Self {
        self.complement(clause)
    }

    pub fn prepositional_adjunct(self, phrase: PrepositionalPhrase) -> Self {
        self.adjunct(phrase)
    }

    pub fn adverbial(self, phrase: AdverbPhrase) -> Self {
        self.adjunct(phrase)
    }

    fn with_clause_features_from(mut self, other: &VerbPhrase) -> Self {
        self.tense = other.tense;
        self.aspect = other.aspect;
        self.polarity = other.polarity;
        self.modal = other.modal;
        self
    }

    fn for_bare_infinitive_embedding(mut self) -> Self {
        self.tense = Tense::Present;
        self.modal = None;
        self.agreement = None;
        self
    }

    fn promote_first_determiner_complement(&mut self) -> Option<DeterminerPhrase> {
        let index = self
            .complements
            .iter()
            .position(|complement| matches!(complement, VerbComplement::DeterminerPhrase(_)))?;

        match self.complements.remove(index) {
            VerbComplement::DeterminerPhrase(phrase) => Some(*phrase),
            _ => unreachable!("checked complement kind before removal"),
        }
    }

    fn render_dependents(&self, parts: &mut Vec<String>) {
        parts.extend(self.complements.iter().map(VerbComplement::render));
        parts.extend(self.adjuncts.iter().map(VerbAdjunct::render));
    }

    fn default_agreement() -> (Person, Number) {
        (Person::Third, Number::Singular)
    }

    fn effective_agreement(&self, fallback: Option<(Person, Number)>) -> (Person, Number) {
        self.agreement
            .clone()
            .or(fallback)
            .unwrap_or_else(Self::default_agreement)
    }

    fn plan(&self) -> VerbPlan {
        match self.modal {
            Some(modal) => self.modal_plan(modal),
            None => self.finite_plan(),
        }
    }

    fn modal_plan(&self, modal: Modal) -> VerbPlan {
        let mut auxiliaries = vec![Auxiliary {
            lemma: AuxLemma::Modal(modal),
            form: AuxForm::Finite,
        }];

        let main_form = match (self.voice, self.aspect) {
            (Voice::Active, Aspect::Simple) => Some(MainForm::Infinitive),
            (Voice::Active, Aspect::Perfect) => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("have")),
                    form: AuxForm::Infinitive,
                });
                Some(MainForm::PastParticiple)
            }
            (Voice::Active, Aspect::Progressive) => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::Infinitive,
                });
                Some(MainForm::PresentParticiple)
            }
            (Voice::Active, Aspect::PerfectProgressive) => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("have")),
                    form: AuxForm::Infinitive,
                });
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::PastParticiple,
                });
                Some(MainForm::PresentParticiple)
            }
            (Voice::Passive, Aspect::Simple) => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::Infinitive,
                });
                Some(MainForm::PastParticiple)
            }
            (Voice::Passive, Aspect::Perfect) => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("have")),
                    form: AuxForm::Infinitive,
                });
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::PastParticiple,
                });
                Some(MainForm::PastParticiple)
            }
            (Voice::Passive, Aspect::Progressive) => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::Infinitive,
                });
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::PresentParticiple,
                });
                Some(MainForm::PastParticiple)
            }
            (Voice::Passive, Aspect::PerfectProgressive) => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("have")),
                    form: AuxForm::Infinitive,
                });
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::PastParticiple,
                });
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::PresentParticiple,
                });
                Some(MainForm::PastParticiple)
            }
        };

        VerbPlan {
            auxiliaries,
            negated: self.polarity == Polarity::Negative,
            main_form,
        }
    }

    fn finite_plan(&self) -> VerbPlan {
        if self.voice == Voice::Passive {
            let mut auxiliaries = match self.aspect {
                Aspect::Simple => vec![Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::Finite,
                }],
                Aspect::Perfect => vec![
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("have")),
                        form: AuxForm::Finite,
                    },
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("be")),
                        form: AuxForm::PastParticiple,
                    },
                ],
                Aspect::Progressive => vec![
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("be")),
                        form: AuxForm::Finite,
                    },
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("be")),
                        form: AuxForm::PresentParticiple,
                    },
                ],
                Aspect::PerfectProgressive => vec![
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("have")),
                        form: AuxForm::Finite,
                    },
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("be")),
                        form: AuxForm::PastParticiple,
                    },
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("be")),
                        form: AuxForm::PresentParticiple,
                    },
                ],
            };

            return VerbPlan {
                auxiliaries: std::mem::take(&mut auxiliaries),
                negated: self.polarity == Polarity::Negative,
                main_form: Some(MainForm::PastParticiple),
            };
        }

        match self.aspect {
            Aspect::Simple if self.polarity == Polarity::Affirmative => VerbPlan {
                auxiliaries: Vec::new(),
                negated: false,
                main_form: Some(MainForm::Finite),
            },
            Aspect::Simple if self.head.infinitive() == "be" => VerbPlan {
                auxiliaries: vec![Auxiliary {
                    lemma: AuxLemma::Verb(self.head.clone()),
                    form: AuxForm::Finite,
                }],
                negated: true,
                main_form: None,
            },
            Aspect::Simple => VerbPlan {
                auxiliaries: vec![Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("do")),
                    form: AuxForm::Finite,
                }],
                negated: true,
                main_form: Some(MainForm::Infinitive),
            },
            Aspect::Perfect => VerbPlan {
                auxiliaries: vec![Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("have")),
                    form: AuxForm::Finite,
                }],
                negated: self.polarity == Polarity::Negative,
                main_form: Some(MainForm::PastParticiple),
            },
            Aspect::Progressive => VerbPlan {
                auxiliaries: vec![Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::Finite,
                }],
                negated: self.polarity == Polarity::Negative,
                main_form: Some(MainForm::PresentParticiple),
            },
            Aspect::PerfectProgressive => VerbPlan {
                auxiliaries: vec![
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("have")),
                        form: AuxForm::Finite,
                    },
                    Auxiliary {
                        lemma: AuxLemma::Verb(Verb::new("be")),
                        form: AuxForm::PastParticiple,
                    },
                ],
                negated: self.polarity == Polarity::Negative,
                main_form: Some(MainForm::PresentParticiple),
            },
        }
    }

    fn finite_form(&self, verb: &Verb, person: &Person, number: &Number, tense: Tense) -> String {
        if verb.infinitive() == "be" {
            return EnglishCore::to_be(
                person,
                number,
                &match tense {
                    Tense::Present => english::Tense::Present,
                    Tense::Past => english::Tense::Past,
                },
                &english::Form::Finite,
            )
            .to_string();
        }

        match tense {
            Tense::Present => {
                if matches!(person, Person::Third) && matches!(number, Number::Singular) {
                    verb.third_person()
                } else {
                    verb.infinitive()
                }
            }
            Tense::Past => verb.past(),
        }
    }

    fn render_verb_form(
        &self,
        verb: &Verb,
        form: MainForm,
        agreement: &(Person, Number),
        tense: Tense,
    ) -> String {
        match form {
            MainForm::Finite => self.finite_form(verb, &agreement.0, &agreement.1, tense),
            MainForm::Infinitive => verb.infinitive(),
            MainForm::PastParticiple => verb.past_participle(),
            MainForm::PresentParticiple => verb.present_participle(),
        }
    }

    fn render_auxiliary(
        &self,
        auxiliary: &Auxiliary,
        agreement: &(Person, Number),
        tense: Tense,
    ) -> String {
        match &auxiliary.lemma {
            AuxLemma::Modal(modal) => modal.render().to_string(),
            AuxLemma::Verb(verb) => match auxiliary.form {
                AuxForm::Finite => self.finite_form(verb, &agreement.0, &agreement.1, tense),
                AuxForm::Infinitive => verb.infinitive(),
                AuxForm::PastParticiple => verb.past_participle(),
                AuxForm::PresentParticiple => verb.present_participle(),
            },
        }
    }

    fn render_with_subject(&self, fallback: Option<(Person, Number)>) -> String {
        let agreement = self.effective_agreement(fallback);
        let plan = self.plan();
        let mut parts = Vec::new();

        if plan.auxiliaries.is_empty() {
            if let Some(main_form) = plan.main_form {
                parts.push(self.render_verb_form(&self.head, main_form, &agreement, self.tense));
            }
        } else {
            for (index, auxiliary) in plan.auxiliaries.iter().enumerate() {
                parts.push(self.render_auxiliary(auxiliary, &agreement, self.tense));
                if index == 0 && plan.negated {
                    parts.push("not".to_string());
                }
            }

            if let Some(main_form) = plan.main_form {
                parts.push(self.render_verb_form(&self.head, main_form, &agreement, self.tense));
            }
        }

        if let Some(particle) = &self.particle {
            parts.push(particle.as_str().to_string());
        }

        self.render_dependents(&mut parts);

        parts.join(" ")
    }

    fn render_nonfinite(&self, form: NonFiniteForm) -> String {
        let mut parts = Vec::new();

        if self.polarity == Polarity::Negative {
            parts.push("not".to_string());
        }

        match form {
            NonFiniteForm::BareInfinitive => match (self.voice, self.aspect) {
                (Voice::Active, Aspect::Simple) => parts.push(self.head.infinitive()),
                (Voice::Active, Aspect::Perfect) => {
                    parts.push("have".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Active, Aspect::Progressive) => {
                    parts.push("be".to_string());
                    parts.push(self.head.present_participle());
                }
                (Voice::Active, Aspect::PerfectProgressive) => {
                    parts.push("have".to_string());
                    parts.push("been".to_string());
                    parts.push(self.head.present_participle());
                }
                (Voice::Passive, Aspect::Simple) => {
                    parts.push("be".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Passive, Aspect::Perfect) => {
                    parts.push("have".to_string());
                    parts.push("been".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Passive, Aspect::Progressive) => {
                    parts.push("be".to_string());
                    parts.push("being".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Passive, Aspect::PerfectProgressive) => {
                    parts.push("have".to_string());
                    parts.push("been".to_string());
                    parts.push("being".to_string());
                    parts.push(self.head.past_participle());
                }
            },
            NonFiniteForm::ToInfinitive => {
                parts.push("to".to_string());
                match (self.voice, self.aspect) {
                    (Voice::Active, Aspect::Simple) => parts.push(self.head.infinitive()),
                    (Voice::Active, Aspect::Perfect) => {
                        parts.push("have".to_string());
                        parts.push(self.head.past_participle());
                    }
                    (Voice::Active, Aspect::Progressive) => {
                        parts.push("be".to_string());
                        parts.push(self.head.present_participle());
                    }
                    (Voice::Active, Aspect::PerfectProgressive) => {
                        parts.push("have".to_string());
                        parts.push("been".to_string());
                        parts.push(self.head.present_participle());
                    }
                    (Voice::Passive, Aspect::Simple) => {
                        parts.push("be".to_string());
                        parts.push(self.head.past_participle());
                    }
                    (Voice::Passive, Aspect::Perfect) => {
                        parts.push("have".to_string());
                        parts.push("been".to_string());
                        parts.push(self.head.past_participle());
                    }
                    (Voice::Passive, Aspect::Progressive) => {
                        parts.push("be".to_string());
                        parts.push("being".to_string());
                        parts.push(self.head.past_participle());
                    }
                    (Voice::Passive, Aspect::PerfectProgressive) => {
                        parts.push("have".to_string());
                        parts.push("been".to_string());
                        parts.push("being".to_string());
                        parts.push(self.head.past_participle());
                    }
                }
            }
            NonFiniteForm::GerundParticiple => match (self.voice, self.aspect) {
                (Voice::Active, Aspect::Simple) => parts.push(self.head.present_participle()),
                (Voice::Active, Aspect::Perfect) => {
                    parts.push("having".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Active, Aspect::Progressive) => {
                    parts.push("being".to_string());
                    parts.push(self.head.present_participle());
                }
                (Voice::Active, Aspect::PerfectProgressive) => {
                    parts.push("having".to_string());
                    parts.push("been".to_string());
                    parts.push(self.head.present_participle());
                }
                (Voice::Passive, Aspect::Simple) => {
                    parts.push("being".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Passive, Aspect::Perfect) => {
                    parts.push("having".to_string());
                    parts.push("been".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Passive, Aspect::Progressive) => {
                    parts.push("being".to_string());
                    parts.push("being".to_string());
                    parts.push(self.head.past_participle());
                }
                (Voice::Passive, Aspect::PerfectProgressive) => {
                    parts.push("having".to_string());
                    parts.push("been".to_string());
                    parts.push("being".to_string());
                    parts.push(self.head.past_participle());
                }
            },
        }

        if let Some(particle) = &self.particle {
            parts.push(particle.clone());
        }

        self.render_dependents(&mut parts);

        parts.join(" ")
    }

    pub fn render(&self) -> String {
        self.render_with_subject(None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhrase {
    light_verb_phrase: LightVerbPhrase,
}

impl TensePhrase {
    pub fn new(subject: DeterminerPhrase, predicate: VerbPhrase) -> Self {
        Self {
            light_verb_phrase: LightVerbPhrase::new(subject, predicate),
        }
    }

    pub fn object(mut self, object: DeterminerPhrase) -> Self {
        let predicate = self.light_verb_phrase.predicate().clone().direct_object(object);
        self.light_verb_phrase = self.light_verb_phrase.with_complement(predicate);
        self
    }

    pub fn prepositional(
        mut self,
        preposition: impl Into<Preposition>,
        object: DeterminerPhrase,
    ) -> Self {
        let predicate = self
            .light_verb_phrase
            .predicate()
            .clone()
            .prepositional_adjunct(PrepositionalPhrase::new(preposition, object));
        self.light_verb_phrase = self.light_verb_phrase.with_complement(predicate);
        self
    }

    pub fn predicate_complement(mut self, complement: AdjPhrase) -> Self {
        let predicate = self
            .light_verb_phrase
            .predicate()
            .clone()
            .predicative_complement(complement);
        self.light_verb_phrase = self.light_verb_phrase.with_complement(predicate);
        self
    }

    pub fn passive(mut self) -> Self {
        let demoted_subject = self.light_verb_phrase.subject().clone();
        let mut predicate = self.light_verb_phrase.predicate().clone();

        if let Some(promoted_subject) = predicate.promote_first_determiner_complement() {
            predicate = predicate.passive().agree_with(&promoted_subject);
            if !demoted_subject.render().is_empty() {
                predicate =
                    predicate.prepositional_adjunct(PrepositionalPhrase::new("by", demoted_subject));
            }
            self.light_verb_phrase = LightVerbPhrase::new(promoted_subject, predicate);
        } else {
            predicate = predicate.passive().agree_with(&demoted_subject);
            self.light_verb_phrase = LightVerbPhrase::new(demoted_subject, predicate);
        }

        self
    }

    pub fn causative(self, causer: DeterminerPhrase) -> Self {
        let causee = self.light_verb_phrase.subject().clone();
        let predicate = self.light_verb_phrase.predicate().clone();
        let embedded =
            NonFiniteClause::bare_infinitive(predicate.clone().for_bare_infinitive_embedding());
        let matrix = VerbPhrase::new("make")
            .with_clause_features_from(&predicate)
            .active()
            .agree_with(&causer)
            .direct_object(causee)
            .non_finite_complement(embedded);

        Self {
            light_verb_phrase: LightVerbPhrase::new(causer, matrix),
        }
    }

    pub fn render(&self) -> String {
        [
            self.light_verb_phrase.subject().render(),
            self.light_verb_phrase
                .predicate()
                .render_with_subject(Some(self.light_verb_phrase.subject().agreement())),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
    }

    pub fn sentence(self) -> Sentence {
        Sentence::new(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonFiniteClause {
    form: NonFiniteForm,
    predicate: VerbPhrase,
    subject: Option<DeterminerPhrase>,
}

impl NonFiniteClause {
    pub fn bare_infinitive(predicate: VerbPhrase) -> Self {
        Self {
            form: NonFiniteForm::BareInfinitive,
            predicate,
            subject: None,
        }
    }

    pub fn to_infinitive(predicate: VerbPhrase) -> Self {
        Self {
            form: NonFiniteForm::ToInfinitive,
            predicate,
            subject: None,
        }
    }

    pub fn gerund_participle(predicate: VerbPhrase) -> Self {
        Self {
            form: NonFiniteForm::GerundParticiple,
            predicate,
            subject: None,
        }
    }

    pub fn subject(mut self, subject: DeterminerPhrase) -> Self {
        self.subject = Some(subject);
        self
    }

    pub fn object(mut self, object: DeterminerPhrase) -> Self {
        self.predicate = self.predicate.direct_object(object);
        self
    }

    pub fn prepositional(
        mut self,
        preposition: impl Into<Preposition>,
        object: DeterminerPhrase,
    ) -> Self {
        self.predicate = self
            .predicate
            .prepositional_adjunct(PrepositionalPhrase::new(preposition, object));
        self
    }

    pub fn predicate_complement(mut self, complement: AdjPhrase) -> Self {
        self.predicate = self.predicate.predicative_complement(complement);
        self
    }

    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        if let Some(subject) = &self.subject {
            let rendered = subject.render();
            if !rendered.is_empty() {
                parts.push(rendered);
            }
        }

        parts.push(self.predicate.render_nonfinite(self.form));
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplementizerPhrase {
    head: Complementizer,
    complement: Box<TensePhrase>,
}

impl ComplementizerPhrase {
    pub fn new(head: Complementizer, complement: TensePhrase) -> Self {
        Self {
            head,
            complement: Box::new(complement),
        }
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

    pub fn render(&self) -> String {
        let head = self.head.render();
        let clause = self.complement.render();
        if head.is_empty() {
            clause
        } else {
            format!("{head} {clause}")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Terminal {
    #[default]
    None,
    Period,
    QuestionMark,
    ExclamationMark,
}

impl Terminal {
    fn render(self) -> &'static str {
        match self {
            Terminal::None => "",
            Terminal::Period => ".",
            Terminal::QuestionMark => "?",
            Terminal::ExclamationMark => "!",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    tense_phrase: TensePhrase,
}

impl Clause {
    pub fn new(subject: DeterminerPhrase, predicate: VerbPhrase) -> Self {
        Self {
            tense_phrase: TensePhrase::new(subject, predicate),
        }
    }

    pub fn from_tense_phrase(tense_phrase: TensePhrase) -> Self {
        Self { tense_phrase }
    }

    pub fn object(mut self, object: DeterminerPhrase) -> Self {
        self.tense_phrase = self.tense_phrase.object(object);
        self
    }

    pub fn prepositional(
        mut self,
        preposition: impl Into<Preposition>,
        object: DeterminerPhrase,
    ) -> Self {
        self.tense_phrase = self.tense_phrase.prepositional(preposition, object);
        self
    }

    pub fn passive(mut self) -> Self {
        self.tense_phrase = self.tense_phrase.passive();
        self
    }

    pub fn causative(mut self, causer: DeterminerPhrase) -> Self {
        self.tense_phrase = self.tense_phrase.causative(causer);
        self
    }

    pub fn render(&self) -> String {
        self.tense_phrase.render()
    }

    pub fn sentence(self) -> Sentence {
        Sentence::new(self)
    }

    pub fn into_tense_phrase(self) -> TensePhrase {
        self.tense_phrase
    }

    pub fn as_tense_phrase(&self) -> &TensePhrase {
        &self.tense_phrase
    }
}

impl From<TensePhrase> for Clause {
    fn from(tense_phrase: TensePhrase) -> Self {
        Self::from_tense_phrase(tense_phrase)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sentence {
    clause: Clause,
    capitalize: bool,
    terminal: Terminal,
}

impl Sentence {
    pub fn new(clause: impl Into<Clause>) -> Self {
        Self {
            clause: clause.into(),
            capitalize: false,
            terminal: Terminal::None,
        }
    }

    pub fn capitalize(mut self) -> Self {
        self.capitalize = true;
        self
    }

    pub fn period(mut self) -> Self {
        self.terminal = Terminal::Period;
        self
    }

    pub fn question_mark(mut self) -> Self {
        self.terminal = Terminal::QuestionMark;
        self
    }

    pub fn exclamation_mark(mut self) -> Self {
        self.terminal = Terminal::ExclamationMark;
        self
    }

    pub fn render(&self) -> String {
        let mut text = self.clause.render();
        if self.capitalize {
            text = capitalize_first(&text);
        }
        text.push_str(self.terminal.render());
        text
    }
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjective_phrase_example() {
        let adj = AdjPhrase::new("bad3")
            .comparative()
            .intensifier(AdverbPhrase::new("far"))
            .complement(PrepositionalPhrase::new(
                "than",
                DeterminerPhrase::proper_name("yesterday"),
            ));

        assert_eq!(adj.render(), "far worse than yesterday");
    }

    #[test]
    fn noun_phrase_example() {
        let np = DeterminerPhrase::new("child")
            .determiner(Determiner::the())
            .modifier(AdjPhrase::new("small").comparative())
            .postmodifier(PrepositionalPhrase::new(
                "from",
                DeterminerPhrase::new("building")
                    .the()
                    .modifier(AdjPhrase::new("next")),
            ))
            .plural();

        assert_eq!(np.render(), "the smaller children from the next building");
    }

    #[test]
    fn verb_phrase_example() {
        let subject = DeterminerPhrase::new("child").the().plural();

        let vp = VerbPhrase::new("eat")
            .present()
            .perfect()
            .negative()
            .agree_with(&subject);

        assert_eq!(vp.render(), "have not eaten");
    }

    #[test]
    fn clause_and_sentence_example() {
        let clause = Clause::new(
            DeterminerPhrase::new("child").the().plural(),
            VerbPhrase::new("steal").past().simple().affirmative(),
        )
        .object(DeterminerPhrase::new("potato").count(7));

        assert_eq!(clause.render(), "the children stole 7 potatoes");

        let sentence = clause.sentence().capitalize().period();
        assert_eq!(sentence.render(), "The children stole 7 potatoes.");
    }
}
