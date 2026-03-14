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
pub struct AdjPhrase {
    adjective: String,
    degree: Option<Degree>,
    intensifier: Option<String>,
    complements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjPhraseError {
    MissingDegree,
}

impl std::fmt::Display for AdjPhraseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            AdjPhraseError::MissingDegree => "adjective phrase is missing a degree",
        };
        f.write_str(message)
    }
}

impl std::error::Error for AdjPhraseError {}

impl AdjPhrase {
    pub fn new(adjective: impl Into<String>) -> Self {
        Self {
            adjective: adjective.into(),
            degree: None,
            intensifier: None,
            complements: Vec::new(),
        }
    }

    pub fn degree(mut self, degree: Degree) -> Self {
        self.degree = Some(degree);
        self
    }

    pub fn intensifier(mut self, intensifier: impl Into<String>) -> Self {
        self.intensifier = Some(intensifier.into());
        self
    }

    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn render(&self) -> Result<String, AdjPhraseError> {
        let degree = self.degree.as_ref().ok_or(AdjPhraseError::MissingDegree)?;
        let mut parts = Vec::new();

        if let Some(intensifier) = &self.intensifier {
            parts.push(intensifier.clone());
        }

        parts.push(English::adj(&self.adjective, degree));

        if !self.complements.is_empty() {
            parts.push(self.complements.join(" "));
        }

        Ok(parts.join(" "))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NounPhrase {
    head: String,
    number: Option<Number>,
    count: Option<u32>,
    determiner: Option<String>,
    modifiers: Vec<NounModifier>,
    complements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NounPhraseError {
    MissingNumber,
    CountNumberMismatch { count: u32, number: Number },
    ModifierPhrase(AdjPhraseError),
}

impl std::fmt::Display for NounPhraseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NounPhraseError::MissingNumber => f.write_str("noun phrase is missing a number"),
            NounPhraseError::CountNumberMismatch { count, number } => {
                write!(
                    f,
                    "noun phrase count {count} conflicts with explicit number {:?}",
                    number
                )
            }
            NounPhraseError::ModifierPhrase(err) => {
                write!(f, "noun phrase has an invalid modifier phrase: {err}")
            }
        }
    }
}

impl std::error::Error for NounPhraseError {}

#[derive(Debug, Clone, PartialEq)]
enum NounModifier {
    Text(String),
    Adjective(AdjPhrase),
}

#[derive(Debug, Clone, PartialEq)]
enum SubjectSpec {
    Agreement(Person, Number),
    NounPhrase(NounPhrase),
}

impl NounPhrase {
    pub fn new<T: Into<Noun>>(noun: T) -> Self {
        let noun = noun.into();
        let mut modifiers = Vec::new();
        let mut complements = Vec::new();

        if let Some(modifier) = noun.modifier {
            modifiers.push(NounModifier::Text(modifier));
        }
        if let Some(complement) = noun.complement {
            complements.push(complement);
        }

        Self {
            head: noun.head,
            number: None,
            count: None,
            determiner: None,
            modifiers,
            complements,
        }
    }

    pub fn number(mut self, number: Number) -> Self {
        self.number = Some(number);
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn determiner(mut self, determiner: impl Into<String>) -> Self {
        self.determiner = Some(determiner.into());
        self
    }

    pub fn modifier(mut self, modifier: impl Into<String>) -> Self {
        self.modifiers.push(NounModifier::Text(modifier.into()));
        self
    }

    pub fn modifier_phrase(mut self, phrase: AdjPhrase) -> Self {
        self.modifiers.push(NounModifier::Adjective(phrase));
        self
    }

    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn render(&self) -> Result<String, NounPhraseError> {
        let number = self.resolve_number()?;
        let noun = self.render_noun()?;

        let mut parts = Vec::new();
        if let Some(determiner) = &self.determiner {
            parts.push(determiner.clone());
        }

        if let Some(count) = self.count {
            parts.push(Noun::count_with_number(noun, count));
        } else {
            parts.push(English::noun(noun, &number));
        }

        Ok(parts.join(" "))
    }

    pub fn agreement(&self) -> Result<(Person, Number), NounPhraseError> {
        Ok((Person::Third, self.resolve_number()?))
    }

    fn resolve_number(&self) -> Result<Number, NounPhraseError> {
        match (self.count, self.number.as_ref()) {
            (Some(count), Some(number)) => {
                let derived = if count == 1 {
                    Number::Singular
                } else {
                    Number::Plural
                };

                if &derived == number {
                    Ok(number.clone())
                } else {
                    Err(NounPhraseError::CountNumberMismatch {
                        count,
                        number: number.clone(),
                    })
                }
            }
            (Some(count), None) => {
                if count == 1 {
                    Ok(Number::Singular)
                } else {
                    Ok(Number::Plural)
                }
            }
            (None, Some(number)) => Ok(number.clone()),
            (None, None) => Err(NounPhraseError::MissingNumber),
        }
    }

    fn render_noun(&self) -> Result<Noun, NounPhraseError> {
        let mut noun = Noun::new(self.head.clone());

        let rendered_modifiers = self
            .modifiers
            .iter()
            .map(|modifier| match modifier {
                NounModifier::Text(text) => Ok(text.clone()),
                NounModifier::Adjective(phrase) => {
                    phrase.render().map_err(NounPhraseError::ModifierPhrase)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        if !rendered_modifiers.is_empty() {
            noun = noun.with_specifier(rendered_modifiers.join(" "));
        }
        if !self.complements.is_empty() {
            noun = noun.with_complement(self.complements.join(" "));
        }

        Ok(noun)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase {
    verb: Verb,
    tense: Option<BaseTense>,
    aspect: Option<Aspect>,
    polarity: Option<Polarity>,
    modal: Option<Modal>,
    subject: Option<SubjectSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerbPhraseError {
    MissingTense,
    MissingAspect,
    MissingPolarity,
    MissingSubject,
    SubjectPhrase(NounPhraseError),
}

impl std::fmt::Display for VerbPhraseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            VerbPhraseError::MissingTense => "verb phrase is missing a tense",
            VerbPhraseError::MissingAspect => "verb phrase is missing an aspect",
            VerbPhraseError::MissingPolarity => "verb phrase is missing a polarity",
            VerbPhraseError::MissingSubject => "verb phrase is missing a subject",
            VerbPhraseError::SubjectPhrase(_) => "verb phrase has an invalid noun phrase subject",
        };
        match self {
            VerbPhraseError::SubjectPhrase(err) => write!(f, "{message}: {err}"),
            _ => f.write_str(message),
        }
    }
}

impl std::error::Error for VerbPhraseError {}

#[derive(Debug, Clone)]
struct RenderContext {
    tense: BaseTense,
    aspect: Aspect,
    polarity: Polarity,
    subject: (Person, Number),
}

impl VerbPhrase {
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

    pub fn tense(mut self, tense: BaseTense) -> Self {
        self.tense = Some(tense);
        self
    }

    pub fn aspect(mut self, aspect: Aspect) -> Self {
        self.aspect = Some(aspect);
        self
    }

    pub fn polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = Some(polarity);
        self
    }

    pub fn modal(mut self, modal: Modal) -> Self {
        self.modal = Some(modal);
        self
    }

    pub fn subject(mut self, person: Person, number: Number) -> Self {
        self.subject = Some(SubjectSpec::Agreement(person, number));
        self
    }

    pub fn subject_noun_phrase(mut self, phrase: &NounPhrase) -> Self {
        self.subject = Some(SubjectSpec::NounPhrase(phrase.clone()));
        self
    }

    pub fn render(&self) -> Result<String, VerbPhraseError> {
        let aspect = self.aspect.ok_or(VerbPhraseError::MissingAspect)?;
        let polarity = self.polarity.ok_or(VerbPhraseError::MissingPolarity)?;

        if let Some(modal) = self.modal {
            return Ok(self.render_with_modal(modal, aspect, polarity));
        }

        let tense = self.tense.ok_or(VerbPhraseError::MissingTense)?;
        let subject = self.resolve_subject()?;

        let context = RenderContext {
            tense,
            aspect,
            polarity,
            subject,
        };

        Ok(match context.aspect {
            Aspect::Simple => self.render_simple(context),
            Aspect::Perfect => {
                self.render_without_modal("have", Verb::past_participle(self.verb.clone()), context)
            }
            Aspect::Progressive => self.render_without_modal(
                "be",
                Verb::present_participle(self.verb.clone()),
                context,
            ),
            Aspect::PerfectProgressive => self.render_without_modal(
                "have",
                format!(
                    "{} {}",
                    Verb::past_participle("be"),
                    Verb::present_participle(self.verb.clone())
                ),
                context,
            ),
        })
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

    fn render_simple(&self, context: RenderContext) -> String {
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
        context: RenderContext,
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

    fn resolve_subject(&self) -> Result<(Person, Number), VerbPhraseError> {
        match self
            .subject
            .as_ref()
            .ok_or(VerbPhraseError::MissingSubject)?
        {
            SubjectSpec::Agreement(person, number) => Ok((person.clone(), number.clone())),
            SubjectSpec::NounPhrase(phrase) => {
                phrase.agreement().map_err(VerbPhraseError::SubjectPhrase)
            }
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
            .render()
            .unwrap();

        assert_eq!(text, "has not eaten");
    }

    #[test]
    fn builder_handles_do_support_for_simple_negatives() {
        let present = VerbPhrase::new("eat")
            .tense(BaseTense::Present)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::Third, Number::Singular)
            .render()
            .unwrap();
        let past = VerbPhrase::new("eat")
            .tense(BaseTense::Past)
            .aspect(Aspect::Simple)
            .polarity(Polarity::Negative)
            .subject(Person::First, Number::Plural)
            .render()
            .unwrap();

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
            .render()
            .unwrap();

        assert_eq!(text, "has given up");
    }

    #[test]
    fn builder_requires_missing_fields_explicitly() {
        assert_eq!(
            VerbPhrase::new("eat").render(),
            Err(VerbPhraseError::MissingAspect)
        );
        assert_eq!(
            VerbPhrase::new("eat")
                .aspect(Aspect::Simple)
                .polarity(Polarity::Affirmative)
                .render(),
            Err(VerbPhraseError::MissingTense)
        );
        assert_eq!(
            VerbPhrase::new("eat")
                .tense(BaseTense::Present)
                .aspect(Aspect::Simple)
                .polarity(Polarity::Affirmative)
                .render(),
            Err(VerbPhraseError::MissingSubject)
        );
    }
}
