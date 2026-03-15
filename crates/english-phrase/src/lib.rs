use english_core::EnglishCore;

pub use english::{Adj, Noun, Number, Person, Verb};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdverbModifier {
    Text(String),
    AdverbPhrase(Box<AdverbPhrase>),
}

impl AdverbModifier {
    fn render(&self) -> String {
        match self {
            AdverbModifier::Text(text) => text.clone(),
            AdverbModifier::AdverbPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for AdverbModifier {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for AdverbModifier {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdverbComplement {
    Text(String),
    PrepositionalPhrase(Box<PrepositionalPhrase>),
}

impl AdverbComplement {
    fn render(&self) -> String {
        match self {
            AdverbComplement::Text(text) => text.clone(),
            AdverbComplement::PrepositionalPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for AdverbComplement {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for AdverbComplement {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<PrepositionalPhrase> for AdverbComplement {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdverbPhrase {
    head: String,
    modifiers: Vec<AdverbModifier>,
    complements: Vec<AdverbComplement>,
}

impl AdverbPhrase {
    pub fn new(head: impl Into<String>) -> Self {
        Self {
            head: head.into(),
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }

    pub fn modifier<M: Into<AdverbModifier>>(mut self, modifier: M) -> Self {
        self.modifiers.push(modifier.into());
        self
    }

    pub fn complement<C: Into<AdverbComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn render(&self) -> String {
        let mut parts = Vec::new();
        parts.extend(self.modifiers.iter().map(AdverbModifier::render));
        parts.push(self.head.clone());
        parts.extend(self.complements.iter().map(AdverbComplement::render));
        parts.join(" ")
    }
}

impl From<AdverbPhrase> for AdverbModifier {
    fn from(phrase: AdverbPhrase) -> Self {
        Self::AdverbPhrase(Box::new(phrase))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepositionalComplement {
    DeterminerPhrase(Box<DeterminerPhrase>),
    AdverbPhrase(Box<AdverbPhrase>),
    Text(String),
}

impl PrepositionalComplement {
    fn render(&self) -> String {
        match self {
            PrepositionalComplement::DeterminerPhrase(phrase) => phrase.render(),
            PrepositionalComplement::AdverbPhrase(phrase) => phrase.render(),
            PrepositionalComplement::Text(text) => text.clone(),
        }
    }
}

impl From<&str> for PrepositionalComplement {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for PrepositionalComplement {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepositionalPhrase {
    preposition: String,
    complement: PrepositionalComplement,
}

impl PrepositionalPhrase {
    pub fn new(
        preposition: impl Into<String>,
        complement: impl Into<PrepositionalComplement>,
    ) -> Self {
        Self {
            preposition: preposition.into(),
            complement: complement.into(),
        }
    }

    pub fn render(&self) -> String {
        format!("{} {}", self.preposition, self.complement.render())
    }
}

impl From<DeterminerPhrase> for PrepositionalComplement {
    fn from(phrase: DeterminerPhrase) -> Self {
        Self::DeterminerPhrase(Box::new(phrase))
    }
}

impl From<AdverbPhrase> for PrepositionalComplement {
    fn from(phrase: AdverbPhrase) -> Self {
        Self::AdverbPhrase(Box::new(phrase))
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
    Text(String),
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

    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
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
            Determiner::Text(text) => text.as_str(),
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

    fn agreement(self) -> (Person, Number) {
        match self {
            Pronoun::I => (Person::First, Number::Singular),
            Pronoun::YouSingular => (Person::Second, Number::Singular),
            Pronoun::YouPlural => (Person::Second, Number::Plural),
            Pronoun::He | Pronoun::She | Pronoun::It => (Person::Third, Number::Singular),
            Pronoun::We => (Person::First, Number::Plural),
            Pronoun::They => (Person::Third, Number::Plural),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intensifier {
    Text(String),
    AdverbPhrase(Box<AdverbPhrase>),
}

impl Intensifier {
    fn render(&self) -> String {
        match self {
            Intensifier::Text(text) => text.clone(),
            Intensifier::AdverbPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for Intensifier {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for Intensifier {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<AdverbPhrase> for Intensifier {
    fn from(phrase: AdverbPhrase) -> Self {
        Self::AdverbPhrase(Box::new(phrase))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Degree {
    #[default]
    Positive,
    Comparative,
    Superlative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjComplement {
    Text(String),
    DeterminerPhrase(Box<DeterminerPhrase>),
    PrepositionalPhrase(Box<PrepositionalPhrase>),
}

impl AdjComplement {
    fn render(&self) -> String {
        match self {
            AdjComplement::Text(text) => text.clone(),
            AdjComplement::DeterminerPhrase(phrase) => phrase.render(),
            AdjComplement::PrepositionalPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for AdjComplement {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for AdjComplement {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<DeterminerPhrase> for AdjComplement {
    fn from(phrase: DeterminerPhrase) -> Self {
        Self::DeterminerPhrase(Box::new(phrase))
    }
}

impl From<PrepositionalPhrase> for AdjComplement {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjPhrase {
    head: Adj,
    degree: Degree,
    intensifier: Option<Intensifier>,
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

    pub fn intensifier<I: Into<Intensifier>>(mut self, intensifier: I) -> Self {
        self.intensifier = Some(intensifier.into());
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NominalModifier {
    Text(String),
    Adjective(AdjPhrase),
}

impl NominalModifier {
    fn render(&self) -> String {
        match self {
            NominalModifier::Text(text) => text.clone(),
            NominalModifier::Adjective(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for NominalModifier {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for NominalModifier {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<AdjPhrase> for NominalModifier {
    fn from(phrase: AdjPhrase) -> Self {
        Self::Adjective(phrase)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NominalComplement {
    Text(String),
    DeterminerPhrase(Box<DeterminerPhrase>),
    PrepositionalPhrase(Box<PrepositionalPhrase>),
}

impl NominalComplement {
    fn render(&self) -> String {
        match self {
            NominalComplement::Text(text) => text.clone(),
            NominalComplement::DeterminerPhrase(phrase) => phrase.render(),
            NominalComplement::PrepositionalPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for NominalComplement {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for NominalComplement {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<DeterminerPhrase> for NominalComplement {
    fn from(phrase: DeterminerPhrase) -> Self {
        Self::DeterminerPhrase(Box::new(phrase))
    }
}

impl From<PrepositionalPhrase> for NominalComplement {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeMarker {
    Who,
    Which,
    That,
    Bare,
    Raw(String),
}

impl RelativeMarker {
    fn render(&self) -> &str {
        match self {
            RelativeMarker::Who => "who",
            RelativeMarker::Which => "which",
            RelativeMarker::That => "that",
            RelativeMarker::Bare => "",
            RelativeMarker::Raw(text) => text.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeClause {
    marker: RelativeMarker,
    body: String,
}

impl RelativeClause {
    pub fn new(marker: RelativeMarker, body: impl Into<String>) -> Self {
        Self {
            marker,
            body: body.into(),
        }
    }

    pub fn who(body: impl Into<String>) -> Self {
        Self::new(RelativeMarker::Who, body)
    }

    pub fn which(body: impl Into<String>) -> Self {
        Self::new(RelativeMarker::Which, body)
    }

    pub fn that(body: impl Into<String>) -> Self {
        Self::new(RelativeMarker::That, body)
    }

    pub fn bare(body: impl Into<String>) -> Self {
        Self::new(RelativeMarker::Bare, body)
    }

    fn render(&self) -> String {
        let marker = self.marker.render();
        if marker.is_empty() {
            self.body.clone()
        } else {
            format!("{marker} {}", self.body)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NominalPostmodifier {
    PrepositionalPhrase(Box<PrepositionalPhrase>),
    RelativeClause(Box<RelativeClause>),
    Raw(String),
}

impl NominalPostmodifier {
    fn render(&self) -> String {
        match self {
            NominalPostmodifier::PrepositionalPhrase(phrase) => phrase.render(),
            NominalPostmodifier::RelativeClause(clause) => clause.render(),
            NominalPostmodifier::Raw(text) => text.clone(),
        }
    }
}

impl From<PrepositionalPhrase> for NominalPostmodifier {
    fn from(phrase: PrepositionalPhrase) -> Self {
        Self::PrepositionalPhrase(Box::new(phrase))
    }
}

impl From<RelativeClause> for NominalPostmodifier {
    fn from(clause: RelativeClause) -> Self {
        Self::RelativeClause(Box::new(clause))
    }
}

impl From<&str> for NominalPostmodifier {
    fn from(text: &str) -> Self {
        Self::Raw(text.to_string())
    }
}

impl From<String> for NominalPostmodifier {
    fn from(text: String) -> Self {
        Self::Raw(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Quantity {
    #[default]
    Singular,
    Plural,
    Count(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominalPhrase {
    head: Noun,
    quantity: Quantity,
    modifiers: Vec<NominalModifier>,
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

    pub fn modifier<M: Into<NominalModifier>>(mut self, modifier: M) -> Self {
        self.modifiers.push(modifier.into());
        self
    }

    pub fn complement<C: Into<NominalComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn postmodifier<M: Into<NominalPostmodifier>>(mut self, postmodifier: M) -> Self {
        self.postmodifiers.push(postmodifier.into());
        self
    }

    pub fn agreement(&self) -> (Person, Number) {
        let number = match self.quantity {
            Quantity::Singular => Number::Singular,
            Quantity::Plural => Number::Plural,
            Quantity::Count(1) => Number::Singular,
            Quantity::Count(_) => Number::Plural,
        };

        (Person::Third, number)
    }

    fn render_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();

        if let Quantity::Count(count) = self.quantity {
            parts.push(count.to_string());
        }

        parts.extend(self.modifiers.iter().map(NominalModifier::render));

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeterminerPhraseCore {
    Nominal(NominalPhrase),
    Pronoun(Pronoun),
    ProperName(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminerPhrase {
    specifier: Option<Box<DeterminerPhrase>>,
    determiner: Option<Determiner>,
    core: DeterminerPhraseCore,
}

impl DeterminerPhrase {
    pub fn new<T: Into<Noun>>(head: T) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::Nominal(NominalPhrase::new(head)),
        }
    }

    pub fn from_nominal(nominal: NominalPhrase) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::Nominal(nominal),
        }
    }

    pub fn pronoun(pronoun: Pronoun) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::Pronoun(pronoun),
        }
    }

    pub fn proper_name(name: impl Into<String>) -> Self {
        Self {
            specifier: None,
            determiner: None,
            core: DeterminerPhraseCore::ProperName(name.into()),
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
        self
    }

    pub fn plural(mut self) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            *nominal = nominal.clone().plural();
        }
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            *nominal = nominal.clone().count(count);
        }
        self
    }

    pub fn modifier<M: Into<NominalModifier>>(mut self, modifier: M) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            nominal.modifiers.push(modifier.into());
        }
        self
    }

    pub fn complement<C: Into<NominalComplement>>(mut self, complement: C) -> Self {
        if let DeterminerPhraseCore::Nominal(nominal) = &mut self.core {
            nominal.complements.push(complement.into());
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
        match &self.core {
            DeterminerPhraseCore::Nominal(nominal) => nominal.agreement(),
            DeterminerPhraseCore::Pronoun(pronoun) => pronoun.agreement(),
            DeterminerPhraseCore::ProperName(_) => (Person::Third, Number::Singular),
        }
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
        parts.join(" ")
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
pub struct VerbPhrase {
    head: Verb,
    tense: Tense,
    aspect: Aspect,
    polarity: Polarity,
    modal: Option<Modal>,
    particle: Option<String>,
    agreement: Option<(Person, Number)>,
}

impl VerbPhrase {
    pub fn new<T: Into<Verb>>(head: T) -> Self {
        Self {
            head: head.into(),
            tense: Tense::Present,
            aspect: Aspect::Simple,
            polarity: Polarity::Affirmative,
            modal: None,
            particle: None,
            agreement: None,
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

        let main_form = match self.aspect {
            Aspect::Simple => Some(MainForm::Infinitive),
            Aspect::Perfect => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("have")),
                    form: AuxForm::Infinitive,
                });
                Some(MainForm::PastParticiple)
            }
            Aspect::Progressive => {
                auxiliaries.push(Auxiliary {
                    lemma: AuxLemma::Verb(Verb::new("be")),
                    form: AuxForm::Infinitive,
                });
                Some(MainForm::PresentParticiple)
            }
            Aspect::PerfectProgressive => {
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
        };

        VerbPlan {
            auxiliaries,
            negated: self.polarity == Polarity::Negative,
            main_form,
        }
    }

    fn finite_plan(&self) -> VerbPlan {
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

        parts.join(" ")
    }

    pub fn render(&self) -> String {
        self.render_with_subject(None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    subject: DeterminerPhrase,
    predicate: VerbPhrase,
    object: Option<DeterminerPhrase>,
    prepositionals: Vec<PrepositionalPhrase>,
}

impl Clause {
    pub fn new(subject: DeterminerPhrase, predicate: VerbPhrase) -> Self {
        Self {
            subject,
            predicate,
            object: None,
            prepositionals: Vec::new(),
        }
    }

    pub fn object(mut self, object: DeterminerPhrase) -> Self {
        self.object = Some(object);
        self
    }

    pub fn prepositional(
        mut self,
        preposition: impl Into<String>,
        object: impl Into<PrepositionalComplement>,
    ) -> Self {
        self.prepositionals
            .push(PrepositionalPhrase::new(preposition, object));
        self
    }

    pub fn render(&self) -> String {
        let mut parts = vec![
            self.subject.render(),
            self.predicate.render_with_subject(Some(self.subject.agreement())),
        ];

        if let Some(object) = &self.object {
            parts.push(object.render());
        }

        parts.extend(
            self.prepositionals
                .iter()
                .map(PrepositionalPhrase::render),
        );

        parts.join(" ")
    }

    pub fn sentence(self) -> Sentence {
        Sentence::new(self)
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
pub struct Sentence {
    clause: Clause,
    capitalize: bool,
    terminal: Terminal,
}

impl Sentence {
    pub fn new(clause: Clause) -> Self {
        Self {
            clause,
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
            .intensifier("far")
            .complement("than yesterday");

        assert_eq!(adj.render(), "far worse than yesterday");
    }

    #[test]
    fn noun_phrase_example() {
        let np = DeterminerPhrase::new("child")
            .determiner(Determiner::the())
            .modifier(AdjPhrase::new("small").comparative())
            .complement("from the next building")
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
