use english::English;

pub use english::{Adj, Degree, Form, Noun, Number, Person, Tense, Verb};

#[doc(hidden)]
pub mod state {
    use super::{Aspect, BaseTense, Degree, Modal, Number, Person, Polarity};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingDegree;

    #[derive(Debug, Clone, PartialEq)]
    pub struct HasDegree(pub(crate) Degree);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingIntensifier;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct HasIntensifier;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingQuantity;

    #[derive(Debug, Clone, PartialEq)]
    pub struct HasQuantity(pub(crate) super::QuantitySpec);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingDeterminer;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct HasDeterminer;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingTense;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HasTense(pub(crate) BaseTense);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingAspect;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HasAspect(pub(crate) Aspect);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingPolarity;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HasPolarity(pub(crate) Polarity);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingSubject;

    #[derive(Debug, Clone, PartialEq)]
    pub struct HasSubject(pub(crate) Person, pub(crate) Number);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MissingModal;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HasModal(pub(crate) Modal);
}

use state::*;

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

#[derive(Debug, Clone, PartialEq)]
struct AdjPhraseData {
    adjective: Adj,
    intensifier: Option<String>,
    complements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedAdjPhrase {
    adjective: Adj,
    degree: Degree,
    intensifier: Option<String>,
    complements: Vec<String>,
}

impl ResolvedAdjPhrase {
    fn render(&self) -> String {
        let mut parts = Vec::new();

        if let Some(intensifier) = &self.intensifier {
            parts.push(intensifier.clone());
        }

        parts.push(English::adj(self.adjective.as_str(), &self.degree));

        if !self.complements.is_empty() {
            parts.push(self.complements.join(" "));
        }

        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjPhrase<DegreeState = MissingDegree, IntensifierState = MissingIntensifier> {
    data: AdjPhraseData,
    degree_state: DegreeState,
    intensifier_state: IntensifierState,
}

impl AdjPhrase<MissingDegree, MissingIntensifier> {
    pub fn new<T: Into<Adj>>(adjective: T) -> Self {
        Self {
            data: AdjPhraseData {
                adjective: adjective.into(),
                intensifier: None,
                complements: Vec::new(),
            },
            degree_state: MissingDegree,
            intensifier_state: MissingIntensifier,
        }
    }
}

impl<IntensifierState> AdjPhrase<MissingDegree, IntensifierState> {
    pub fn degree(self, degree: Degree) -> AdjPhrase<HasDegree, IntensifierState> {
        AdjPhrase {
            data: self.data,
            degree_state: HasDegree(degree),
            intensifier_state: self.intensifier_state,
        }
    }
}

impl<DegreeState> AdjPhrase<DegreeState, MissingIntensifier> {
    pub fn intensifier(self, intensifier: impl Into<String>) -> AdjPhrase<DegreeState, HasIntensifier> {
        let mut data = self.data;
        data.intensifier = Some(intensifier.into());
        AdjPhrase {
            data,
            degree_state: self.degree_state,
            intensifier_state: HasIntensifier,
        }
    }
}

impl<DegreeState, IntensifierState> AdjPhrase<DegreeState, IntensifierState> {
    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.data.complements.push(complement.into());
        self
    }
}

impl<IntensifierState> AdjPhrase<HasDegree, IntensifierState> {
    fn resolve(&self) -> ResolvedAdjPhrase {
        ResolvedAdjPhrase {
            adjective: self.data.adjective.clone(),
            degree: self.degree_state.0.clone(),
            intensifier: self.data.intensifier.clone(),
            complements: self.data.complements.clone(),
        }
    }

    pub fn render(&self) -> String {
        self.resolve().render()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum QuantitySpec {
    Number(Number),
    Count(u32),
}

#[derive(Debug, Clone, PartialEq)]
enum NounModifier {
    Text(String),
    Adjective(ResolvedAdjPhrase),
}

impl NounModifier {
    fn render(&self) -> String {
        match self {
            NounModifier::Text(text) => text.clone(),
            NounModifier::Adjective(phrase) => phrase.render(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum NounComplement {
    Text(String),
}

impl NounComplement {
    fn render(&self) -> String {
        match self {
            NounComplement::Text(text) => text.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NounPhraseData {
    head: Noun,
    determiner: Option<String>,
    modifiers: Vec<NounModifier>,
    complements: Vec<NounComplement>,
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedNounPhrase {
    head: Noun,
    modifiers: Vec<NounModifier>,
    complements: Vec<NounComplement>,
    quantity: QuantitySpec,
    determiner: Option<String>,
}

impl ResolvedNounPhrase {
    fn render(&self) -> String {
        let mut parts = Vec::new();
        if let Some(determiner) = &self.determiner {
            parts.push(determiner.clone());
        }

        parts.extend(self.modifiers.iter().map(NounModifier::render));

        match &self.quantity {
            QuantitySpec::Number(number) => parts.push(English::noun(self.head.as_str(), number)),
            QuantitySpec::Count(count) => parts.push(self.head.count_with_number(*count)),
        }

        if !self.complements.is_empty() {
            parts.push(
                self.complements
                    .iter()
                    .map(NounComplement::render)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }

        parts.join(" ")
    }

    fn agreement(&self) -> (Person, Number) {
        let number = match &self.quantity {
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct NounPhrase<QuantityState = MissingQuantity, DeterminerState = MissingDeterminer> {
    data: NounPhraseData,
    quantity_state: QuantityState,
    determiner_state: DeterminerState,
}

impl NounPhrase<MissingQuantity, MissingDeterminer> {
    pub fn new<T: Into<Noun>>(noun: T) -> Self {
        let noun = noun.into();

        Self {
            data: NounPhraseData {
                head: noun,
                determiner: None,
                modifiers: Vec::new(),
                complements: Vec::new(),
            },
            quantity_state: MissingQuantity,
            determiner_state: MissingDeterminer,
        }
    }
}

impl<DeterminerState> NounPhrase<MissingQuantity, DeterminerState> {
    pub fn number(self, number: Number) -> NounPhrase<HasQuantity, DeterminerState> {
        NounPhrase {
            data: self.data,
            quantity_state: HasQuantity(QuantitySpec::Number(number)),
            determiner_state: self.determiner_state,
        }
    }

    pub fn singular(self) -> NounPhrase<HasQuantity, DeterminerState> {
        self.number(Number::Singular)
    }

    pub fn plural(self) -> NounPhrase<HasQuantity, DeterminerState> {
        self.number(Number::Plural)
    }

    pub fn count(self, count: u32) -> NounPhrase<HasQuantity, DeterminerState> {
        NounPhrase {
            data: self.data,
            quantity_state: HasQuantity(QuantitySpec::Count(count)),
            determiner_state: self.determiner_state,
        }
    }
}

impl<QuantityState> NounPhrase<QuantityState, MissingDeterminer> {
    pub fn determiner(self, determiner: impl Into<String>) -> NounPhrase<QuantityState, HasDeterminer> {
        let mut data = self.data;
        data.determiner = Some(determiner.into());
        NounPhrase {
            data,
            quantity_state: self.quantity_state,
            determiner_state: HasDeterminer,
        }
    }
}

impl<QuantityState, DeterminerState> NounPhrase<QuantityState, DeterminerState> {
    pub fn modifier(mut self, modifier: impl Into<String>) -> Self {
        self.data.modifiers.push(NounModifier::Text(modifier.into()));
        self
    }

    pub fn modifier_phrase<IntensifierState>(
        mut self,
        phrase: AdjPhrase<HasDegree, IntensifierState>,
    ) -> Self {
        self.data
            .modifiers
            .push(NounModifier::Adjective(phrase.resolve()));
        self
    }

    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.data
            .complements
            .push(NounComplement::Text(complement.into()));
        self
    }
}

impl<DeterminerState> NounPhrase<HasQuantity, DeterminerState> {
    fn resolve(&self) -> ResolvedNounPhrase {
        ResolvedNounPhrase {
            head: self.data.head.clone(),
            modifiers: self.data.modifiers.clone(),
            complements: self.data.complements.clone(),
            quantity: self.quantity_state.0.clone(),
            determiner: self.data.determiner.clone(),
        }
    }

    pub fn render(&self) -> String {
        self.resolve().render()
    }

    pub fn agreement(&self) -> (Person, Number) {
        self.resolve().agreement()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct VerbPhraseData {
    head: Verb,
    particle: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedFiniteVerbPhrase {
    head: Verb,
    particle: Option<String>,
    tense: BaseTense,
    aspect: Aspect,
    polarity: Polarity,
    subject: (Person, Number),
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedModalVerbPhrase {
    head: Verb,
    particle: Option<String>,
    modal: Modal,
    aspect: Aspect,
    polarity: Polarity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase<
    TenseState = MissingTense,
    AspectState = MissingAspect,
    PolarityState = MissingPolarity,
    SubjectState = MissingSubject,
    ModalState = MissingModal,
> {
    data: VerbPhraseData,
    tense_state: TenseState,
    aspect_state: AspectState,
    polarity_state: PolarityState,
    subject_state: SubjectState,
    modal_state: ModalState,
}

#[derive(Debug, Clone)]
struct FiniteRenderContext {
    tense: BaseTense,
    aspect: Aspect,
    polarity: Polarity,
    subject: (Person, Number),
}

impl VerbPhrase<MissingTense, MissingAspect, MissingPolarity, MissingSubject, MissingModal> {
    pub fn new<T: Into<Verb>>(verb: T) -> Self {
        let verb = verb.into();
        Self {
            data: VerbPhraseData {
                head: verb,
                particle: None,
            },
            tense_state: MissingTense,
            aspect_state: MissingAspect,
            polarity_state: MissingPolarity,
            subject_state: MissingSubject,
            modal_state: MissingModal,
        }
    }
}

impl<TenseState, AspectState, PolarityState, SubjectState, ModalState>
    VerbPhrase<TenseState, AspectState, PolarityState, SubjectState, ModalState>
{
    pub fn particle(mut self, particle: impl Into<String>) -> Self {
        self.data.particle = Some(particle.into());
        self
    }

    fn with_particle(&self, head: String, particle: &Option<String>) -> String {
        if let Some(particle) = particle {
            format!("{head} {particle}")
        } else {
            head
        }
    }
}

impl<AspectState, PolarityState, SubjectState>
    VerbPhrase<MissingTense, AspectState, PolarityState, SubjectState, MissingModal>
{
    pub fn tense(
        self,
        tense: BaseTense,
    ) -> VerbPhrase<HasTense, AspectState, PolarityState, SubjectState, MissingModal> {
        VerbPhrase {
            data: self.data,
            tense_state: HasTense(tense),
            aspect_state: self.aspect_state,
            polarity_state: self.polarity_state,
            subject_state: self.subject_state,
            modal_state: MissingModal,
        }
    }
}

impl<TenseState, PolarityState, SubjectState, ModalState>
    VerbPhrase<TenseState, MissingAspect, PolarityState, SubjectState, ModalState>
{
    pub fn aspect(
        self,
        aspect: Aspect,
    ) -> VerbPhrase<TenseState, HasAspect, PolarityState, SubjectState, ModalState> {
        VerbPhrase {
            data: self.data,
            tense_state: self.tense_state,
            aspect_state: HasAspect(aspect),
            polarity_state: self.polarity_state,
            subject_state: self.subject_state,
            modal_state: self.modal_state,
        }
    }
}

impl<TenseState, AspectState, SubjectState, ModalState>
    VerbPhrase<TenseState, AspectState, MissingPolarity, SubjectState, ModalState>
{
    pub fn polarity(
        self,
        polarity: Polarity,
    ) -> VerbPhrase<TenseState, AspectState, HasPolarity, SubjectState, ModalState> {
        VerbPhrase {
            data: self.data,
            tense_state: self.tense_state,
            aspect_state: self.aspect_state,
            polarity_state: HasPolarity(polarity),
            subject_state: self.subject_state,
            modal_state: self.modal_state,
        }
    }
}

impl<TenseState, AspectState, PolarityState>
    VerbPhrase<TenseState, AspectState, PolarityState, MissingSubject, MissingModal>
{
    pub fn subject(
        self,
        person: Person,
        number: Number,
    ) -> VerbPhrase<TenseState, AspectState, PolarityState, HasSubject, MissingModal> {
        VerbPhrase {
            data: self.data,
            tense_state: self.tense_state,
            aspect_state: self.aspect_state,
            polarity_state: self.polarity_state,
            subject_state: HasSubject(person, number),
            modal_state: MissingModal,
        }
    }

    pub fn subject_noun_phrase<DeterminerState>(
        self,
        phrase: &NounPhrase<HasQuantity, DeterminerState>,
    ) -> VerbPhrase<TenseState, AspectState, PolarityState, HasSubject, MissingModal> {
        let (person, number) = phrase.agreement();
        self.subject(person, number)
    }
}

impl<TenseState, AspectState, PolarityState, SubjectState>
    VerbPhrase<TenseState, AspectState, PolarityState, SubjectState, MissingModal>
{
    pub fn modal(
        self,
        modal: Modal,
    ) -> VerbPhrase<MissingTense, AspectState, PolarityState, MissingSubject, HasModal> {
        VerbPhrase {
            data: self.data,
            tense_state: MissingTense,
            aspect_state: self.aspect_state,
            polarity_state: self.polarity_state,
            subject_state: MissingSubject,
            modal_state: HasModal(modal),
        }
    }
}

impl<TenseState, AspectState, PolarityState, SubjectState, ModalState>
    VerbPhrase<TenseState, AspectState, PolarityState, SubjectState, ModalState>
{
    fn render_modal(&self, phrase: ResolvedModalVerbPhrase) -> String {
        let mut chunks = vec![phrase.modal.as_str().to_string()];

        if phrase.polarity == Polarity::Negative {
            chunks.push("not".to_string());
        }

        match phrase.aspect {
            Aspect::Simple => {
                let head = phrase.head.infinitive();
                chunks.push(self.with_particle(head, &phrase.particle));
            }
            Aspect::Perfect => {
                chunks.push("have".to_string());
                let head = phrase.head.past_participle();
                chunks.push(self.with_particle(head, &phrase.particle));
            }
            Aspect::Progressive => {
                chunks.push("be".to_string());
                let head = phrase.head.present_participle();
                chunks.push(self.with_particle(head, &phrase.particle));
            }
            Aspect::PerfectProgressive => {
                chunks.push("have".to_string());
                chunks.push(Verb::new("be").past_participle());
                let head = phrase.head.present_participle();
                chunks.push(self.with_particle(head, &phrase.particle));
            }
        }

        chunks.join(" ")
    }

    fn render_finite(&self, phrase: ResolvedFiniteVerbPhrase) -> String {
        let context = FiniteRenderContext {
            tense: phrase.tense,
            aspect: phrase.aspect,
            polarity: phrase.polarity,
            subject: phrase.subject,
        };

        match context.aspect {
            Aspect::Simple => self.render_simple(&context, phrase.head.as_str(), &phrase.particle),
            Aspect::Perfect => self.render_without_modal(
                "have",
                self.with_particle(phrase.head.past_participle(), &phrase.particle),
                &context,
            ),
            Aspect::Progressive => self.render_without_modal(
                "be",
                self.with_particle(phrase.head.present_participle(), &phrase.particle),
                &context,
            ),
            Aspect::PerfectProgressive => self.render_without_modal(
                "have",
                format!(
                    "{} {}",
                    Verb::new("be").past_participle(),
                    self.with_particle(phrase.head.present_participle(), &phrase.particle)
                ),
                &context,
            ),
        }
    }

    fn render_simple(
        &self,
        context: &FiniteRenderContext,
        head: &str,
        particle: &Option<String>,
    ) -> String {
        let (person, number) = (&context.subject.0, &context.subject.1);

        if context.polarity == Polarity::Affirmative {
            let finite = English::verb(head, person, number, &Self::low_level_tense(context.tense), &Form::Finite);
            return self.with_particle(finite, particle);
        }

        if Verb::new(head).infinitive() == "be" {
            let be = English::verb(head, person, number, &Self::low_level_tense(context.tense), &Form::Finite);
            return self.with_particle(format!("{be} not"), particle);
        }

        let do_aux = English::verb("do", person, number, &Self::low_level_tense(context.tense), &Form::Finite);
        let main = self.with_particle(Verb::new(head).infinitive(), particle);
        format!("{do_aux} not {main}")
    }

    fn render_without_modal(
        &self,
        auxiliary: &str,
        tail: String,
        context: &FiniteRenderContext,
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

impl VerbPhrase<HasTense, HasAspect, HasPolarity, HasSubject, MissingModal> {
    fn resolve_finite(&self) -> ResolvedFiniteVerbPhrase {
        ResolvedFiniteVerbPhrase {
            head: self.data.head.clone(),
            particle: self.data.particle.clone(),
            tense: self.tense_state.0,
            aspect: self.aspect_state.0,
            polarity: self.polarity_state.0,
            subject: (self.subject_state.0.clone(), self.subject_state.1.clone()),
        }
    }

    pub fn render(&self) -> String {
        self.render_finite(self.resolve_finite())
    }
}

impl<TenseState, SubjectState> VerbPhrase<TenseState, HasAspect, HasPolarity, SubjectState, HasModal> {
    fn resolve_modal(&self) -> ResolvedModalVerbPhrase {
        ResolvedModalVerbPhrase {
            head: self.data.head.clone(),
            particle: self.data.particle.clone(),
            modal: self.modal_state.0,
            aspect: self.aspect_state.0,
            polarity: self.polarity_state.0,
        }
    }

    pub fn render(&self) -> String {
        self.render_modal(self.resolve_modal())
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
        let text = VerbPhrase::new("give")
            .particle("up")
            .tense(BaseTense::Present)
            .aspect(Aspect::Perfect)
            .polarity(Polarity::Affirmative)
            .subject(Person::Third, Number::Singular)
            .render();

        assert_eq!(text, "has given up");
    }
}
