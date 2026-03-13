use english::English;

pub use english::{Form, Number, Person, Tense, Verb};

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
pub struct VerbPhrase {
    verb: Verb,
    tense: Option<BaseTense>,
    aspect: Option<Aspect>,
    polarity: Option<Polarity>,
    modal: Option<Modal>,
    subject: Option<(Person, Number)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerbPhraseError {
    MissingTense,
    MissingAspect,
    MissingPolarity,
    MissingSubject,
}

impl std::fmt::Display for VerbPhraseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            VerbPhraseError::MissingTense => "verb phrase is missing a tense",
            VerbPhraseError::MissingAspect => "verb phrase is missing an aspect",
            VerbPhraseError::MissingPolarity => "verb phrase is missing a polarity",
            VerbPhraseError::MissingSubject => "verb phrase is missing a subject",
        };
        f.write_str(message)
    }
}

impl std::error::Error for VerbPhraseError {}

type SubjectRef<'a> = (&'a Person, &'a Number);

#[derive(Debug, Clone, Copy)]
struct RenderContext<'a> {
    tense: BaseTense,
    aspect: Aspect,
    polarity: Polarity,
    subject: SubjectRef<'a>,
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
        self.subject = Some((person, number));
        self
    }

    pub fn render(&self) -> Result<String, VerbPhraseError> {
        let aspect = self.aspect.ok_or(VerbPhraseError::MissingAspect)?;
        let polarity = self.polarity.ok_or(VerbPhraseError::MissingPolarity)?;

        if let Some(modal) = self.modal {
            return Ok(self.render_with_modal(modal, aspect, polarity));
        }

        let tense = self.tense.ok_or(VerbPhraseError::MissingTense)?;
        let subject = self
            .subject
            .as_ref()
            .ok_or(VerbPhraseError::MissingSubject)?;

        let context = RenderContext {
            tense,
            aspect,
            polarity,
            subject: (&subject.0, &subject.1),
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

    fn render_simple(&self, context: RenderContext<'_>) -> String {
        let (person, number) = context.subject;

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
        context: RenderContext<'_>,
    ) -> String {
        let (person, number) = context.subject;
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
