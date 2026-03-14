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
pub enum ModalTense {
    Present,
    Preterite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    Will(ModalTense),
    Can(ModalTense),
    Shall(ModalTense),
    May(ModalTense),
    Must,
}

impl Modal {
    fn as_str(self) -> &'static str {
        match self {
            Modal::Will(ModalTense::Present) => "will",
            Modal::Will(ModalTense::Preterite) => "would",
            Modal::Can(ModalTense::Present) => "can",
            Modal::Can(ModalTense::Preterite) => "could",
            Modal::Shall(ModalTense::Present) => "shall",
            Modal::Shall(ModalTense::Preterite) => "should",
            Modal::May(ModalTense::Present) => "may",
            Modal::May(ModalTense::Preterite) => "might",
            Modal::Must => "must",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissingDegree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HasDegree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissingQuantity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HasQuantity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissingTense;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HasTense;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissingAspect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HasAspect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissingPolarity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HasPolarity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissingSubject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HasSubject;

#[derive(Debug, Clone, PartialEq)]
struct AdjPhraseData {
    adjective: String,
    degree: Option<Degree>,
    intensifier: Option<String>,
    complements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjPhrase<DegreeState = MissingDegree> {
    data: AdjPhraseData,
    degree_state: DegreeState,
}

impl AdjPhrase<MissingDegree> {
    pub fn new(adjective: impl Into<String>) -> Self {
        Self {
            data: AdjPhraseData {
                adjective: adjective.into(),
                degree: None,
                intensifier: None,
                complements: Vec::new(),
            },
            degree_state: MissingDegree,
        }
    }

    pub fn degree(self, degree: Degree) -> AdjPhrase<HasDegree> {
        let mut data = self.data;
        data.degree = Some(degree);
        AdjPhrase {
            data,
            degree_state: HasDegree,
        }
    }
}

impl<DegreeState> AdjPhrase<DegreeState> {
    pub fn intensifier(mut self, intensifier: impl Into<String>) -> Self {
        self.data.intensifier = Some(intensifier.into());
        self
    }

    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.data.complements.push(complement.into());
        self
    }
}

impl AdjPhrase<HasDegree> {
    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        if let Some(intensifier) = &self.data.intensifier {
            parts.push(intensifier.clone());
        }

        parts.push(English::adj(
            &self.data.adjective,
            self.data.degree.as_ref().expect("degree set by typestate"),
        ));

        if !self.data.complements.is_empty() {
            parts.push(self.data.complements.join(" "));
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
struct NounPhraseData {
    head: String,
    quantity: Option<QuantitySpec>,
    determiner: Option<String>,
    modifiers: Vec<String>,
    complements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NounPhrase<QuantityState = MissingQuantity> {
    data: NounPhraseData,
    quantity_state: QuantityState,
}

impl NounPhrase<MissingQuantity> {
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
            data: NounPhraseData {
                head: noun.head,
                quantity: None,
                determiner: None,
                modifiers,
                complements,
            },
            quantity_state: MissingQuantity,
        }
    }

    pub fn number(self, number: Number) -> NounPhrase<HasQuantity> {
        let mut data = self.data;
        data.quantity = Some(QuantitySpec::Number(number));
        NounPhrase {
            data,
            quantity_state: HasQuantity,
        }
    }

    pub fn singular(self) -> NounPhrase<HasQuantity> {
        self.number(Number::Singular)
    }

    pub fn plural(self) -> NounPhrase<HasQuantity> {
        self.number(Number::Plural)
    }

    pub fn count(self, count: u32) -> NounPhrase<HasQuantity> {
        let mut data = self.data;
        data.quantity = Some(QuantitySpec::Count(count));
        NounPhrase {
            data,
            quantity_state: HasQuantity,
        }
    }
}

impl<QuantityState> NounPhrase<QuantityState> {
    pub fn determiner(mut self, determiner: impl Into<String>) -> Self {
        self.data.determiner = Some(determiner.into());
        self
    }

    pub fn modifier(mut self, modifier: impl Into<String>) -> Self {
        self.data.modifiers.push(modifier.into());
        self
    }

    pub fn modifier_phrase(mut self, phrase: AdjPhrase<HasDegree>) -> Self {
        self.data.modifiers.push(phrase.render());
        self
    }

    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.data.complements.push(complement.into());
        self
    }
}

impl NounPhrase<HasQuantity> {
    pub fn render(&self) -> String {
        let noun = self.to_english_noun();

        let mut parts = Vec::new();
        if let Some(determiner) = &self.data.determiner {
            parts.push(determiner.clone());
        }

        match self
            .data
            .quantity
            .as_ref()
            .expect("quantity set by typestate")
        {
            QuantitySpec::Number(number) => parts.push(English::noun(noun, number)),
            QuantitySpec::Count(count) => parts.push(Noun::count_with_number(noun, *count)),
        }

        parts.join(" ")
    }

    pub fn agreement(&self) -> (Person, Number) {
        let number = match self
            .data
            .quantity
            .as_ref()
            .expect("quantity set by typestate")
        {
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
        let mut noun = Noun::new(self.data.head.clone());

        if !self.data.modifiers.is_empty() {
            noun = noun.with_specifier(self.data.modifiers.join(" "));
        }
        if !self.data.complements.is_empty() {
            noun = noun.with_complement(self.data.complements.join(" "));
        }

        noun
    }
}

#[derive(Debug, Clone, PartialEq)]
struct VerbPhraseData {
    verb: Verb,
    tense: Option<BaseTense>,
    aspect: Option<Aspect>,
    polarity: Option<Polarity>,
    modal: Option<Modal>,
    subject: Option<(Person, Number)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase<
    TenseState = MissingTense,
    AspectState = MissingAspect,
    PolarityState = MissingPolarity,
    SubjectState = MissingSubject,
> {
    data: VerbPhraseData,
    tense_state: TenseState,
    aspect_state: AspectState,
    polarity_state: PolarityState,
    subject_state: SubjectState,
}

#[derive(Debug, Clone)]
struct RenderContext {
    tense: BaseTense,
    aspect: Aspect,
    polarity: Polarity,
    subject: (Person, Number),
}

impl VerbPhrase<MissingTense, MissingAspect, MissingPolarity, MissingSubject> {
    pub fn new<T: Into<Verb>>(verb: T) -> Self {
        Self {
            data: VerbPhraseData {
                verb: verb.into(),
                tense: None,
                aspect: None,
                polarity: None,
                modal: None,
                subject: None,
            },
            tense_state: MissingTense,
            aspect_state: MissingAspect,
            polarity_state: MissingPolarity,
            subject_state: MissingSubject,
        }
    }
}

impl<AspectState, PolarityState, SubjectState>
    VerbPhrase<MissingTense, AspectState, PolarityState, SubjectState>
{
    pub fn tense(
        self,
        tense: BaseTense,
    ) -> VerbPhrase<HasTense, AspectState, PolarityState, SubjectState> {
        let mut data = self.data;
        data.tense = Some(tense);
        VerbPhrase {
            data,
            tense_state: HasTense,
            aspect_state: self.aspect_state,
            polarity_state: self.polarity_state,
            subject_state: self.subject_state,
        }
    }
}

impl<TenseState, PolarityState, SubjectState>
    VerbPhrase<TenseState, MissingAspect, PolarityState, SubjectState>
{
    pub fn aspect(
        self,
        aspect: Aspect,
    ) -> VerbPhrase<TenseState, HasAspect, PolarityState, SubjectState> {
        let mut data = self.data;
        data.aspect = Some(aspect);
        VerbPhrase {
            data,
            tense_state: self.tense_state,
            aspect_state: HasAspect,
            polarity_state: self.polarity_state,
            subject_state: self.subject_state,
        }
    }
}

impl<TenseState, AspectState, SubjectState>
    VerbPhrase<TenseState, AspectState, MissingPolarity, SubjectState>
{
    pub fn polarity(
        self,
        polarity: Polarity,
    ) -> VerbPhrase<TenseState, AspectState, HasPolarity, SubjectState> {
        let mut data = self.data;
        data.polarity = Some(polarity);
        VerbPhrase {
            data,
            tense_state: self.tense_state,
            aspect_state: self.aspect_state,
            polarity_state: HasPolarity,
            subject_state: self.subject_state,
        }
    }
}

impl<TenseState, AspectState, PolarityState>
    VerbPhrase<TenseState, AspectState, PolarityState, MissingSubject>
{
    pub fn subject(
        self,
        person: Person,
        number: Number,
    ) -> VerbPhrase<TenseState, AspectState, PolarityState, HasSubject> {
        let mut data = self.data;
        data.subject = Some((person, number));
        VerbPhrase {
            data,
            tense_state: self.tense_state,
            aspect_state: self.aspect_state,
            polarity_state: self.polarity_state,
            subject_state: HasSubject,
        }
    }

    pub fn subject_noun_phrase(
        self,
        phrase: &NounPhrase<HasQuantity>,
    ) -> VerbPhrase<TenseState, AspectState, PolarityState, HasSubject> {
        let (person, number) = phrase.agreement();
        self.subject(person, number)
    }
}

impl<TenseState, AspectState, PolarityState, SubjectState>
    VerbPhrase<TenseState, AspectState, PolarityState, SubjectState>
{
    pub fn modal(mut self, modal: Modal) -> Self {
        self.data.modal = Some(modal);
        self
    }
}

impl VerbPhrase<HasTense, HasAspect, HasPolarity, HasSubject> {
    pub fn render(&self) -> String {
        let context = RenderContext {
            tense: self.data.tense.expect("tense set by typestate"),
            aspect: self.data.aspect.expect("aspect set by typestate"),
            polarity: self.data.polarity.expect("polarity set by typestate"),
            subject: self.data.subject.clone().expect("subject set by typestate"),
        };

        if let Some(modal) = self.data.modal {
            return self.render_with_modal(modal, context.aspect, context.polarity);
        }

        match context.aspect {
            Aspect::Simple => self.render_simple(&context),
            Aspect::Perfect => self.render_without_modal(
                "have",
                Verb::past_participle(self.data.verb.clone()),
                &context,
            ),
            Aspect::Progressive => self.render_without_modal(
                "be",
                Verb::present_participle(self.data.verb.clone()),
                &context,
            ),
            Aspect::PerfectProgressive => self.render_without_modal(
                "have",
                format!(
                    "{} {}",
                    Verb::past_participle("be"),
                    Verb::present_participle(self.data.verb.clone())
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
            Aspect::Simple => chunks.push(Verb::infinitive(self.data.verb.clone())),
            Aspect::Perfect => {
                chunks.push("have".to_string());
                chunks.push(Verb::past_participle(self.data.verb.clone()));
            }
            Aspect::Progressive => {
                chunks.push("be".to_string());
                chunks.push(Verb::present_participle(self.data.verb.clone()));
            }
            Aspect::PerfectProgressive => {
                chunks.push("have".to_string());
                chunks.push(Verb::past_participle("be"));
                chunks.push(Verb::present_participle(self.data.verb.clone()));
            }
        }

        chunks.join(" ")
    }

    fn render_simple(&self, context: &RenderContext) -> String {
        let (person, number) = (&context.subject.0, &context.subject.1);

        if context.polarity == Polarity::Affirmative {
            return English::verb(
                self.data.verb.clone(),
                person,
                number,
                &Self::low_level_tense(context.tense),
                &Form::Finite,
            );
        }

        if Verb::infinitive(self.data.verb.clone()) == "be" {
            let be = English::verb(
                self.data.verb.clone(),
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
        format!("{do_aux} not {}", Verb::infinitive(self.data.verb.clone()))
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
