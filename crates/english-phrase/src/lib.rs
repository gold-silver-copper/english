use english_core::EnglishCore;

pub use english::{Adj, Noun, Number, Person, Verb};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text(String);

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self(text)
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
    Text(Text),
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

    pub fn text(text: impl Into<Text>) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intensifier {
    Text(Text),
}

impl Intensifier {
    fn render(&self) -> &str {
        match self {
            Intensifier::Text(text) => text.as_str(),
        }
    }
}

impl From<&str> for Intensifier {
    fn from(text: &str) -> Self {
        Self::Text(text.into())
    }
}

impl From<String> for Intensifier {
    fn from(text: String) -> Self {
        Self::Text(text.into())
    }
}

impl From<Text> for Intensifier {
    fn from(text: Text) -> Self {
        Self::Text(text)
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
    Text(Text),
    NounPhrase(Box<NounPhrase>),
}

impl AdjComplement {
    fn render(&self) -> String {
        match self {
            AdjComplement::Text(text) => text.as_str().to_string(),
            AdjComplement::NounPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for AdjComplement {
    fn from(text: &str) -> Self {
        Self::Text(text.into())
    }
}

impl From<String> for AdjComplement {
    fn from(text: String) -> Self {
        Self::Text(text.into())
    }
}

impl From<Text> for AdjComplement {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl From<NounPhrase> for AdjComplement {
    fn from(phrase: NounPhrase) -> Self {
        Self::NounPhrase(Box::new(phrase))
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
            parts.push(intensifier.render().to_string());
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
pub enum NounModifier {
    Text(Text),
    Adjective(AdjPhrase),
}

impl NounModifier {
    fn render(&self) -> String {
        match self {
            NounModifier::Text(text) => text.as_str().to_string(),
            NounModifier::Adjective(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for NounModifier {
    fn from(text: &str) -> Self {
        Self::Text(text.into())
    }
}

impl From<String> for NounModifier {
    fn from(text: String) -> Self {
        Self::Text(text.into())
    }
}

impl From<Text> for NounModifier {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl From<AdjPhrase> for NounModifier {
    fn from(phrase: AdjPhrase) -> Self {
        Self::Adjective(phrase)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NounComplement {
    Text(Text),
    NounPhrase(Box<NounPhrase>),
}

impl NounComplement {
    fn render(&self) -> String {
        match self {
            NounComplement::Text(text) => text.as_str().to_string(),
            NounComplement::NounPhrase(phrase) => phrase.render(),
        }
    }
}

impl From<&str> for NounComplement {
    fn from(text: &str) -> Self {
        Self::Text(text.into())
    }
}

impl From<String> for NounComplement {
    fn from(text: String) -> Self {
        Self::Text(text.into())
    }
}

impl From<Text> for NounComplement {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl From<NounPhrase> for NounComplement {
    fn from(phrase: NounPhrase) -> Self {
        Self::NounPhrase(Box::new(phrase))
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
pub struct NounPhrase {
    head: Noun,
    quantity: Quantity,
    determiner: Option<Determiner>,
    modifiers: Vec<NounModifier>,
    complements: Vec<NounComplement>,
}

impl NounPhrase {
    pub fn new<T: Into<Noun>>(head: T) -> Self {
        Self {
            head: head.into(),
            quantity: Quantity::Singular,
            determiner: None,
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }

    pub fn determiner(mut self, determiner: Determiner) -> Self {
        self.determiner = Some(determiner);
        self
    }

    pub fn the(self) -> Self {
        self.determiner(Determiner::the())
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

    pub fn modifier<M: Into<NounModifier>>(mut self, modifier: M) -> Self {
        self.modifiers.push(modifier.into());
        self
    }

    pub fn complement<C: Into<NounComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
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

    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        if let Some(determiner) = &self.determiner {
            parts.push(determiner.render().to_string());
        }

        if let Quantity::Count(count) = self.quantity {
            parts.push(count.to_string());
        }

        parts.extend(self.modifiers.iter().map(NounModifier::render));

        let head = match self.quantity {
            Quantity::Singular => self.head.singular(),
            Quantity::Plural => self.head.plural(),
            Quantity::Count(count) => self.head.count(count),
        };
        parts.push(head);

        parts.extend(self.complements.iter().map(NounComplement::render));

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PrepositionalPhrase {
    preposition: Text,
    object: NounPhrase,
}

impl PrepositionalPhrase {
    fn render(&self) -> String {
        format!("{} {}", self.preposition.as_str(), self.object.render())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase {
    head: Verb,
    tense: Tense,
    aspect: Aspect,
    polarity: Polarity,
    modal: Option<Modal>,
    particle: Option<Text>,
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

    pub fn particle(mut self, particle: impl Into<Text>) -> Self {
        self.particle = Some(particle.into());
        self
    }

    pub fn subject(mut self, person: Person, number: Number) -> Self {
        self.agreement = Some((person, number));
        self
    }

    pub fn agree_with(mut self, subject: &NounPhrase) -> Self {
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
    subject: NounPhrase,
    predicate: VerbPhrase,
    object: Option<NounPhrase>,
    prepositionals: Vec<PrepositionalPhrase>,
}

impl Clause {
    pub fn new(subject: NounPhrase, predicate: VerbPhrase) -> Self {
        Self {
            subject,
            predicate,
            object: None,
            prepositionals: Vec::new(),
        }
    }

    pub fn object(mut self, object: NounPhrase) -> Self {
        self.object = Some(object);
        self
    }

    pub fn prepositional(
        mut self,
        preposition: impl Into<Text>,
        object: NounPhrase,
    ) -> Self {
        self.prepositionals.push(PrepositionalPhrase {
            preposition: preposition.into(),
            object,
        });
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
        let np = NounPhrase::new("child")
            .determiner(Determiner::the())
            .modifier(AdjPhrase::new("small").comparative())
            .complement("from the next building")
            .plural();

        assert_eq!(np.render(), "the smaller children from the next building");
    }

    #[test]
    fn verb_phrase_example() {
        let subject = NounPhrase::new("child").the().plural();

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
            NounPhrase::new("child").the().plural(),
            VerbPhrase::new("steal").past().simple().affirmative(),
        )
        .object(NounPhrase::new("potato").count(7));

        assert_eq!(clause.render(), "the children stole 7 potatoes");

        let sentence = clause.sentence().capitalize().period();
        assert_eq!(sentence.render(), "The children stole 7 potatoes.");
    }
}
