use english::English;

pub use english::{Adj, Degree, Form, Noun, Number, Person, Tense, Verb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseTense {
    Present,
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aspect {
    Simple,
    Perfect,
    Progressive,
    PerfectProgressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Affirmative,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    Will,
    Would,
    Could,
    Can,
    Should,
}

impl Modal {
    fn as_str(self) -> &'static str {
        match self {
            Modal::Will => "will",
            Modal::Would => "would",
            Modal::Could => "could",
            Modal::Can => "can",
            Modal::Should => "should",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjPhrase<const HAS_DEGREE: bool = false> {
    adjective: String,
    degree: Option<Degree>,
    intensifier: Option<String>,
    complements: Vec<String>,
}

impl AdjPhrase<false> {
    pub fn new(adjective: impl Into<String>) -> Self {
        Self {
            adjective: adjective.into(),
            degree: None,
            intensifier: None,
            complements: Vec::new(),
        }
    }

    pub fn degree(self, degree: Degree) -> AdjPhrase<true> {
        AdjPhrase {
            adjective: self.adjective,
            degree: Some(degree),
            intensifier: self.intensifier,
            complements: self.complements,
        }
    }
}

impl<const HAS_DEGREE: bool> AdjPhrase<HAS_DEGREE> {
    pub fn intensifier(mut self, intensifier: impl Into<String>) -> Self {
        self.intensifier = Some(intensifier.into());
        self
    }

    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.complements.push(complement.into());
        self
    }
}

impl AdjPhrase<true> {
    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        if let Some(intensifier) = &self.intensifier {
            parts.push(intensifier.clone());
        }

        parts.push(English::adj(
            &self.adjective,
            self.degree.as_ref().expect("degree set by typestate"),
        ));

        if !self.complements.is_empty() {
            parts.push(self.complements.join(" "));
        }

        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
enum QuantitySpec {
    Number(Number),
    Count(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NounPhrase<const HAS_QUANTITY: bool = false> {
    head: String,
    quantity: Option<QuantitySpec>,
    determiner: Option<String>,
    modifiers: Vec<String>,
    complements: Vec<String>,
}

impl NounPhrase<false> {
    pub fn new<T: Into<Noun>>(noun: T) -> Self {
        let noun = noun.into();
        let mut modifiers = Vec::new();
        let mut complements = Vec::new();

        if let Some(modifier) = noun.modifier {
            modifiers.push(modifier);
        }
        if let Some(complement) = noun.complement {
            complements.push(complement);
        }

        Self {
            head: noun.head,
            quantity: None,
            determiner: None,
            modifiers,
            complements,
        }
    }

    pub fn number(self, number: Number) -> NounPhrase<true> {
        NounPhrase {
            head: self.head,
            quantity: Some(QuantitySpec::Number(number)),
            determiner: self.determiner,
            modifiers: self.modifiers,
            complements: self.complements,
        }
    }

    pub fn singular(self) -> NounPhrase<true> {
        self.number(Number::Singular)
    }

    pub fn plural(self) -> NounPhrase<true> {
        self.number(Number::Plural)
    }

    pub fn count(self, count: u32) -> NounPhrase<true> {
        NounPhrase {
            head: self.head,
            quantity: Some(QuantitySpec::Count(count)),
            determiner: self.determiner,
            modifiers: self.modifiers,
            complements: self.complements,
        }
    }
}

impl<const HAS_QUANTITY: bool> NounPhrase<HAS_QUANTITY> {
    pub fn determiner(mut self, determiner: impl Into<String>) -> Self {
        self.determiner = Some(determiner.into());
        self
    }

    pub fn modifier(mut self, modifier: impl Into<String>) -> Self {
        self.modifiers.push(modifier.into());
        self
    }

    pub fn modifier_phrase(mut self, phrase: AdjPhrase<true>) -> Self {
        self.modifiers.push(phrase.render());
        self
    }

    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.complements.push(complement.into());
        self
    }
}

impl NounPhrase<true> {
    pub fn render(&self) -> String {
        let noun = self.to_english_noun();

        let mut parts = Vec::new();
        if let Some(determiner) = &self.determiner {
            parts.push(determiner.clone());
        }

        match self.quantity.as_ref().expect("quantity set by typestate") {
            QuantitySpec::Number(number) => parts.push(English::noun(noun, number)),
            QuantitySpec::Count(count) => parts.push(Noun::count_with_number(noun, *count)),
        }

        parts.join(" ")
    }

    pub fn agreement(&self) -> (Person, Number) {
        let number = match self.quantity.as_ref().expect("quantity set by typestate") {
            QuantitySpec::Number(number) => number.clone(),
            QuantitySpec::Count(count) => {
                if *count == 1 {
                    Number::Singular
                } else {
                    Number::Plural
                }
            }
        };

        (Person::Third, number)
    }

    fn to_english_noun(&self) -> Noun {
        let mut noun = Noun::new(self.head.clone());

        if !self.modifiers.is_empty() {
            noun = noun.with_specifier(self.modifiers.join(" "));
        }
        if !self.complements.is_empty() {
            noun = noun.with_complement(self.complements.join(" "));
        }

        noun
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase<
    const HAS_TENSE: bool = false,
    const HAS_ASPECT: bool = false,
    const HAS_POLARITY: bool = false,
    const HAS_SUBJECT: bool = false,
> {
    verb: Verb,
    tense: Option<BaseTense>,
    aspect: Option<Aspect>,
    polarity: Option<Polarity>,
    modal: Option<Modal>,
    subject: Option<(Person, Number)>,
}

#[derive(Debug, Clone)]
struct RenderContext {
    tense: BaseTense,
    aspect: Aspect,
    polarity: Polarity,
    subject: (Person, Number),
}

impl VerbPhrase<false, false, false, false> {
    pub fn new<T: Into<Verb>>(verb: T) -> Self {
        Self {
            verb: verb.into(),
            tense: None,
            aspect: None,
            polarity: None,
            modal: None,
            subject: None,
        }
    }
}

impl<const HAS_ASPECT: bool, const HAS_POLARITY: bool, const HAS_SUBJECT: bool>
    VerbPhrase<false, HAS_ASPECT, HAS_POLARITY, HAS_SUBJECT>
{
    pub fn tense(
        self,
        tense: BaseTense,
    ) -> VerbPhrase<true, HAS_ASPECT, HAS_POLARITY, HAS_SUBJECT> {
        VerbPhrase {
            verb: self.verb,
            tense: Some(tense),
            aspect: self.aspect,
            polarity: self.polarity,
            modal: self.modal,
            subject: self.subject,
        }
    }
}

impl<const HAS_TENSE: bool, const HAS_POLARITY: bool, const HAS_SUBJECT: bool>
    VerbPhrase<HAS_TENSE, false, HAS_POLARITY, HAS_SUBJECT>
{
    pub fn aspect(self, aspect: Aspect) -> VerbPhrase<HAS_TENSE, true, HAS_POLARITY, HAS_SUBJECT> {
        VerbPhrase {
            verb: self.verb,
            tense: self.tense,
            aspect: Some(aspect),
            polarity: self.polarity,
            modal: self.modal,
            subject: self.subject,
        }
    }
}

impl<const HAS_TENSE: bool, const HAS_ASPECT: bool, const HAS_SUBJECT: bool>
    VerbPhrase<HAS_TENSE, HAS_ASPECT, false, HAS_SUBJECT>
{
    pub fn polarity(
        self,
        polarity: Polarity,
    ) -> VerbPhrase<HAS_TENSE, HAS_ASPECT, true, HAS_SUBJECT> {
        VerbPhrase {
            verb: self.verb,
            tense: self.tense,
            aspect: self.aspect,
            polarity: Some(polarity),
            modal: self.modal,
            subject: self.subject,
        }
    }
}

impl<const HAS_TENSE: bool, const HAS_ASPECT: bool, const HAS_POLARITY: bool>
    VerbPhrase<HAS_TENSE, HAS_ASPECT, HAS_POLARITY, false>
{
    pub fn subject(
        self,
        person: Person,
        number: Number,
    ) -> VerbPhrase<HAS_TENSE, HAS_ASPECT, HAS_POLARITY, true> {
        VerbPhrase {
            verb: self.verb,
            tense: self.tense,
            aspect: self.aspect,
            polarity: self.polarity,
            modal: self.modal,
            subject: Some((person, number)),
        }
    }

    pub fn subject_noun_phrase(
        self,
        phrase: &NounPhrase<true>,
    ) -> VerbPhrase<HAS_TENSE, HAS_ASPECT, HAS_POLARITY, true> {
        let (person, number) = phrase.agreement();
        self.subject(person, number)
    }
}

impl<
    const HAS_TENSE: bool,
    const HAS_ASPECT: bool,
    const HAS_POLARITY: bool,
    const HAS_SUBJECT: bool,
> VerbPhrase<HAS_TENSE, HAS_ASPECT, HAS_POLARITY, HAS_SUBJECT>
{
    pub fn modal(mut self, modal: Modal) -> Self {
        self.modal = Some(modal);
        self
    }
}

impl VerbPhrase<true, true, true, true> {
    pub fn render(&self) -> String {
        let context = RenderContext {
            tense: self.tense.expect("tense set by typestate"),
            aspect: self.aspect.expect("aspect set by typestate"),
            polarity: self.polarity.expect("polarity set by typestate"),
            subject: self.subject.clone().expect("subject set by typestate"),
        };

        if let Some(modal) = self.modal {
            return self.render_with_modal(modal, context.aspect, context.polarity);
        }

        match context.aspect {
            Aspect::Simple => self.render_simple(&context),
            Aspect::Perfect => self.render_without_modal(
                "have",
                Verb::past_participle(self.verb.clone()),
                &context,
            ),
            Aspect::Progressive => self.render_without_modal(
                "be",
                Verb::present_participle(self.verb.clone()),
                &context,
            ),
            Aspect::PerfectProgressive => self.render_without_modal(
                "have",
                format!(
                    "{} {}",
                    Verb::past_participle("be"),
                    Verb::present_participle(self.verb.clone())
                ),
                &context,
            ),
        }
    }

    fn render_with_modal(&self, modal: Modal, aspect: Aspect, polarity: Polarity) -> String {
        let mut chunks = vec![modal.as_str().to_string()];

        if polarity == Polarity::Negative {
            chunks.push("not".to_string());
        }

        match aspect {
            Aspect::Simple => chunks.push(Verb::infinitive(self.verb.clone())),
            Aspect::Perfect => {
                chunks.push("have".to_string());
                chunks.push(Verb::past_participle(self.verb.clone()));
            }
            Aspect::Progressive => {
                chunks.push("be".to_string());
                chunks.push(Verb::present_participle(self.verb.clone()));
            }
            Aspect::PerfectProgressive => {
                chunks.push("have".to_string());
                chunks.push(Verb::past_participle("be"));
                chunks.push(Verb::present_participle(self.verb.clone()));
            }
        }

        chunks.join(" ")
    }

    fn render_simple(&self, context: &RenderContext) -> String {
        let (person, number) = (&context.subject.0, &context.subject.1);

        if context.polarity == Polarity::Affirmative {
            return English::verb(
                self.verb.clone(),
                person,
                number,
                &Self::low_level_tense(context.tense),
                &Form::Finite,
            );
        }

        if Verb::infinitive(self.verb.clone()) == "be" {
            let be = English::verb(
                self.verb.clone(),
                person,
                number,
                &Self::low_level_tense(context.tense),
                &Form::Finite,
            );
            return format!("{be} not");
        }

        let do_aux = English::verb(
            "do",
            person,
            number,
            &Self::low_level_tense(context.tense),
            &Form::Finite,
        );
        format!("{do_aux} not {}", Verb::infinitive(self.verb.clone()))
    }

    fn render_without_modal(
        &self,
        auxiliary: &str,
        tail: String,
        context: &RenderContext,
    ) -> String {
        let (person, number) = (&context.subject.0, &context.subject.1);
        let finite_aux = English::verb(
            auxiliary,
            person,
            number,
            &Self::low_level_tense(context.tense),
            &Form::Finite,
        );

        if context.polarity == Polarity::Negative {
            format!("{finite_aux} not {tail}")
        } else {
            format!("{finite_aux} {tail}")
        }
    }

    fn low_level_tense(tense: BaseTense) -> Tense {
        match tense {
            BaseTense::Present => Tense::Present,
            BaseTense::Past => Tense::Past,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_renders_negative_perfect() {
        let text = VerbPhrase::new("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render();

        assert_eq!(text, "has not eaten");
    }

    #[test]
    fn builder_handles_do_support_for_simple_negatives() {
        let present = VerbPhrase::new("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render();
        let past = VerbPhrase::new("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render();

        assert_eq!(present, "does not eat");
        assert_eq!(past, "did not eat");
    }

    #[test]
    fn builder_preserves_phrasal_verbs() {
        let give_up = Verb::new("give").with_particle("up");
        let text = VerbPhrase::new(give_up)
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render();

        assert_eq!(text, "has given up");
    }
}
