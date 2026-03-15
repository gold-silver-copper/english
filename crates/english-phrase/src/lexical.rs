use english::{Adj, Noun, Verb};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Countability {
    Count,
    Mass,
    ProperName,
    PronounLike,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Definiteness {
    Definite,
    Indefinite,
    Demonstrative,
    ProperName,
    PronounLike,
    Bare,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LexicalAnimacy {
    Animate,
    Inanimate,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdverbEntry(String);

impl AdverbEntry {
    pub fn new(head: impl Into<String>) -> Self {
        Self(head.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for AdverbEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AdverbEntry {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Particle(String);

impl Particle {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Particle {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Particle {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepositionEntry {
    head: String,
    complements: Vec<ComplementCategory>,
}

impl PrepositionEntry {
    pub fn new(head: impl Into<String>) -> Self {
        Self {
            head: head.into(),
            complements: vec![ComplementCategory::DeterminerPhrase],
        }
    }

    pub fn with_complement(mut self, category: ComplementCategory) -> Self {
        if !self.complements.contains(&category) {
            self.complements.push(category);
        }
        self
    }

    pub fn as_str(&self) -> &str {
        &self.head
    }

    pub fn complements(&self) -> &[ComplementCategory] {
        &self.complements
    }
}

impl From<&str> for PrepositionEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for PrepositionEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determiner {
    The,
    A,
    An,
    This,
    That,
    These,
    Those,
}

impl Determiner {
    pub fn render(self) -> &'static str {
        match self {
            Determiner::The => "the",
            Determiner::A => "a",
            Determiner::An => "an",
            Determiner::This => "this",
            Determiner::That => "that",
            Determiner::These => "these",
            Determiner::Those => "those",
        }
    }

    pub fn definiteness(self) -> Definiteness {
        match self {
            Determiner::The => Definiteness::Definite,
            Determiner::A | Determiner::An => Definiteness::Indefinite,
            Determiner::This | Determiner::That | Determiner::These | Determiner::Those => {
                Definiteness::Demonstrative
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminerEntry {
    head: Determiner,
}

impl DeterminerEntry {
    pub fn new(head: Determiner) -> Self {
        Self { head }
    }

    pub fn head(&self) -> Determiner {
        self.head
    }
}

impl From<Determiner> for DeterminerEntry {
    fn from(value: Determiner) -> Self {
        Self::new(value)
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
    pub fn render(self) -> &'static str {
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

    pub fn possessive_determiner(self) -> &'static str {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeMarker {
    Who,
    Which,
    That,
    Bare,
}

impl RelativeMarker {
    pub fn render(self) -> &'static str {
        match self {
            RelativeMarker::Who => "who",
            RelativeMarker::Which => "which",
            RelativeMarker::That => "that",
            RelativeMarker::Bare => "",
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

    pub fn render(&self) -> &str {
        match self {
            Complementizer::That => "that",
            Complementizer::Null => "",
            Complementizer::Relative(marker) => marker.render(),
            Complementizer::Custom(text) => text.as_str(),
        }
    }
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
    pub fn render(self) -> &'static str {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounEntry {
    lemma: Noun,
    countability: Countability,
    default_animacy: LexicalAnimacy,
}

impl NounEntry {
    pub fn new(lemma: impl Into<Noun>) -> Self {
        Self {
            lemma: lemma.into(),
            countability: Countability::Count,
            default_animacy: LexicalAnimacy::Unknown,
        }
    }

    pub fn mass(lemma: impl Into<Noun>) -> Self {
        Self::new(lemma).with_countability(Countability::Mass)
    }

    pub fn animate(lemma: impl Into<Noun>) -> Self {
        Self::new(lemma).with_animacy(LexicalAnimacy::Animate)
    }

    pub fn inanimate(lemma: impl Into<Noun>) -> Self {
        Self::new(lemma).with_animacy(LexicalAnimacy::Inanimate)
    }

    pub fn with_countability(mut self, countability: Countability) -> Self {
        self.countability = countability;
        self
    }

    pub fn with_animacy(mut self, animacy: LexicalAnimacy) -> Self {
        self.default_animacy = animacy;
        self
    }

    pub fn lemma(&self) -> &Noun {
        &self.lemma
    }

    pub fn countability(&self) -> Countability {
        self.countability
    }

    pub fn default_animacy(&self) -> LexicalAnimacy {
        self.default_animacy
    }
}

impl From<&str> for NounEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for NounEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjectiveEntry {
    lemma: Adj,
}

impl AdjectiveEntry {
    pub fn new(lemma: impl Into<Adj>) -> Self {
        Self {
            lemma: lemma.into(),
        }
    }

    pub fn lemma(&self) -> &Adj {
        &self.lemma
    }
}

impl From<&str> for AdjectiveEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AdjectiveEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplementCategory {
    DeterminerPhrase,
    ComplementizerPhrase,
    NonFiniteClause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateCategory {
    Adjective,
    DeterminerPhrase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseKind {
    Finite,
    BareInfinitive,
    ToInfinitive,
    GerundParticiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    Forbidden,
    Optional,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObliqueSelection {
    preposition: Option<String>,
    required: bool,
}

impl ObliqueSelection {
    pub fn any_optional() -> Self {
        Self {
            preposition: None,
            required: false,
        }
    }

    pub fn required(preposition: impl Into<String>) -> Self {
        Self {
            preposition: Some(preposition.into()),
            required: true,
        }
    }

    pub fn optional(preposition: impl Into<String>) -> Self {
        Self {
            preposition: Some(preposition.into()),
            required: false,
        }
    }

    pub fn matches(&self, preposition: &str) -> bool {
        match &self.preposition {
            Some(expected) => expected == preposition,
            None => true,
        }
    }

    pub fn required_flag(&self) -> bool {
        self.required
    }

    pub fn preposition(&self) -> Option<&str> {
        self.preposition.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbSelection {
    direct_object: Cardinality,
    indirect_object: Cardinality,
    obliques: Vec<ObliqueSelection>,
    predicate_categories: Vec<PredicateCategory>,
    clausal_categories: Vec<ClauseKind>,
    passive_allowed: bool,
}

impl VerbSelection {
    pub fn open() -> Self {
        Self {
            direct_object: Cardinality::Optional,
            indirect_object: Cardinality::Optional,
            obliques: vec![ObliqueSelection::any_optional()],
            predicate_categories: vec![
                PredicateCategory::Adjective,
                PredicateCategory::DeterminerPhrase,
            ],
            clausal_categories: vec![
                ClauseKind::Finite,
                ClauseKind::BareInfinitive,
                ClauseKind::ToInfinitive,
                ClauseKind::GerundParticiple,
            ],
            passive_allowed: true,
        }
    }

    pub fn intransitive() -> Self {
        Self {
            direct_object: Cardinality::Forbidden,
            indirect_object: Cardinality::Forbidden,
            obliques: vec![ObliqueSelection::any_optional()],
            predicate_categories: Vec::new(),
            clausal_categories: Vec::new(),
            passive_allowed: false,
        }
    }

    pub fn transitive() -> Self {
        Self {
            direct_object: Cardinality::Required,
            indirect_object: Cardinality::Forbidden,
            obliques: vec![ObliqueSelection::any_optional()],
            predicate_categories: Vec::new(),
            clausal_categories: Vec::new(),
            passive_allowed: true,
        }
    }

    pub fn ditransitive() -> Self {
        Self {
            direct_object: Cardinality::Required,
            indirect_object: Cardinality::Required,
            obliques: vec![ObliqueSelection::any_optional()],
            predicate_categories: Vec::new(),
            clausal_categories: Vec::new(),
            passive_allowed: true,
        }
    }

    pub fn copular() -> Self {
        Self {
            direct_object: Cardinality::Forbidden,
            indirect_object: Cardinality::Forbidden,
            obliques: vec![ObliqueSelection::any_optional()],
            predicate_categories: vec![
                PredicateCategory::Adjective,
                PredicateCategory::DeterminerPhrase,
            ],
            clausal_categories: vec![ClauseKind::ToInfinitive, ClauseKind::Finite],
            passive_allowed: false,
        }
    }

    pub fn with_direct_object(mut self, direct_object: Cardinality) -> Self {
        self.direct_object = direct_object;
        self
    }

    pub fn with_indirect_object(mut self, indirect_object: Cardinality) -> Self {
        self.indirect_object = indirect_object;
        self
    }

    pub fn with_oblique(mut self, oblique: ObliqueSelection) -> Self {
        self.obliques.push(oblique);
        self
    }

    pub fn with_predicate_category(mut self, category: PredicateCategory) -> Self {
        if !self.predicate_categories.contains(&category) {
            self.predicate_categories.push(category);
        }
        self
    }

    pub fn with_clause_kind(mut self, kind: ClauseKind) -> Self {
        if !self.clausal_categories.contains(&kind) {
            self.clausal_categories.push(kind);
        }
        self
    }

    pub fn with_passive_allowed(mut self, allowed: bool) -> Self {
        self.passive_allowed = allowed;
        self
    }

    pub fn direct_object(&self) -> Cardinality {
        self.direct_object
    }

    pub fn indirect_object(&self) -> Cardinality {
        self.indirect_object
    }

    pub fn obliques(&self) -> &[ObliqueSelection] {
        &self.obliques
    }

    pub fn predicate_categories(&self) -> &[PredicateCategory] {
        &self.predicate_categories
    }

    pub fn clausal_categories(&self) -> &[ClauseKind] {
        &self.clausal_categories
    }

    pub fn passive_allowed(&self) -> bool {
        self.passive_allowed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbEntry {
    lemma: Verb,
    selection: VerbSelection,
}

impl VerbEntry {
    pub fn new(lemma: impl Into<Verb>) -> Self {
        Self {
            lemma: lemma.into(),
            selection: VerbSelection::open(),
        }
    }

    pub fn intransitive(lemma: impl Into<Verb>) -> Self {
        Self {
            lemma: lemma.into(),
            selection: VerbSelection::intransitive(),
        }
    }

    pub fn transitive(lemma: impl Into<Verb>) -> Self {
        Self {
            lemma: lemma.into(),
            selection: VerbSelection::transitive(),
        }
    }

    pub fn ditransitive(lemma: impl Into<Verb>) -> Self {
        Self {
            lemma: lemma.into(),
            selection: VerbSelection::ditransitive(),
        }
    }

    pub fn copular(lemma: impl Into<Verb>) -> Self {
        Self {
            lemma: lemma.into(),
            selection: VerbSelection::copular(),
        }
    }

    pub fn with_selection(mut self, selection: VerbSelection) -> Self {
        self.selection = selection;
        self
    }

    pub fn lemma(&self) -> &Verb {
        &self.lemma
    }

    pub fn selection(&self) -> &VerbSelection {
        &self.selection
    }

    pub fn as_str(&self) -> &str {
        self.lemma.as_str()
    }
}

impl From<&str> for VerbEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for VerbEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
