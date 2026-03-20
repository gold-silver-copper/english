use english::*;

type EntityId = usize;

#[derive(Clone, Copy)]
enum Determiner {
    Definite,
    Indefinite,
    Bare,
}

#[derive(Clone, Copy)]
struct PrepPhrase {
    prep: &'static str,
    object: EntityId,
}

#[derive(Clone)]
struct GameAdj {
    lemma: &'static str,
    degree: Degree,
    complements: Vec<PrepPhrase>,
}

#[derive(Clone)]
struct GameAdv {
    lemma: &'static str,
    modifier: Option<&'static str>,
}

#[derive(Clone)]
struct GameNoun {
    lemma: &'static str,
    proper_name: Option<&'static str>,
    determiner: Determiner,
    modifiers: Vec<GameAdj>,
    complements: Vec<PrepPhrase>,
}

#[derive(Clone)]
enum VerbComplement {
    Object(EntityId),
    Prep(PrepPhrase),
}

#[derive(Clone)]
struct GameVerb {
    lemma: &'static str,
    modifiers: Vec<GameAdv>,
    complements: Vec<VerbComplement>,
    adjuncts: Vec<PrepPhrase>,
}

#[derive(Clone)]
struct Entity {
    noun: GameNoun,
    number: Number,
    person: Person,
    gender: Gender,
    animacy: Animacy,
}

struct ActionTriple {
    actor: EntityId,
    predicate: GameVerb,
}

#[derive(Clone, Copy)]
struct Viewpoint {
    tense: Tense,
    speaker: Option<EntityId>,
    listener: Option<EntityId>,
}

#[derive(Clone, Copy)]
enum MentionStyle {
    DeicticOrDefinite,
    Introduce,
}

#[derive(Default)]
struct World {
    entities: Vec<Entity>,
}

fn main() {
    // This example shows one minimal but extensible pattern for procedural game
    // text. `english` handles inflection only. The domain structs here carry
    // modifiers, complements, adjuncts, and viewpoint for semantic triples.

    let mut world = World::default();

    let raid = world.add(thing("raid"));
    let swamp = world.add(thing("swamp"));
    let gate = world.add(thing("gate").modifier(adj("ruined")));
    let spear = world.add(thing("spear").modifier(adj("rusty")));
    let goblin = world.add(
        creature("goblin")
            .modifier(adj("blue"))
            .modifier(adj("responsible").complement("for", raid))
            .complement("from", swamp),
    );
    let orc = world.add(creature("orc").modifier(adj("yellow")));
    let guard = world.add(named("Captain Mira", Gender::Feminine));

    let attack = action(
        goblin,
        verb("attack")
            .modifier(adv("quickly").modified_by("very"))
            .object(orc)
            .complement("with", spear)
            .adjunct("near", gate),
    );

    let neutral_present = world.simple_sentence(&attack, Viewpoint::present());
    let neutral_past = world.simple_sentence(&attack, Viewpoint::past());
    let victim_view =
        world.simple_sentence(&attack, Viewpoint::present().speaker(orc).listener(guard));
    let witness_view = world.witness_sentence(
        guard,
        &attack,
        Viewpoint::past().speaker(guard).listener(orc),
    );
    let worse = world.adjective(
        &adj("bad3").degree(Degree::Comparative),
        Viewpoint::present(),
    );

    assert_eq!(
        world.mention(
            goblin,
            Case::Nominative,
            Viewpoint::present(),
            MentionStyle::DeicticOrDefinite,
        ),
        "the blue goblin responsible for the raid from the swamp"
    );
    assert_eq!(
        neutral_present,
        "The blue goblin responsible for the raid from the swamp very quickly attacks the yellow orc with the rusty spear near the ruined gate."
    );
    assert_eq!(
        neutral_past,
        "The blue goblin responsible for the raid from the swamp very quickly attacked the yellow orc with the rusty spear near the ruined gate."
    );
    assert_eq!(
        victim_view,
        "The blue goblin responsible for the raid from the swamp very quickly attacks me with the rusty spear near the ruined gate."
    );
    assert_eq!(
        witness_view,
        "I saw a blue goblin responsible for the raid from the swamp very quickly attack a yellow orc with a rusty spear near the ruined gate."
    );
    assert_eq!(worse, "worse");

    println!("{neutral_present}");
    println!("{neutral_past}");
    println!("{victim_view}");
    println!("{witness_view}");
}

fn thing(lemma: &'static str) -> Entity {
    Entity::common(lemma, Gender::Neuter, Animacy::Inanimate)
}

fn creature(lemma: &'static str) -> Entity {
    Entity::common(lemma, Gender::Neuter, Animacy::Animate)
}

fn named(name: &'static str, gender: Gender) -> Entity {
    Entity::proper(name, gender, Animacy::Animate)
}

fn adj(lemma: &'static str) -> GameAdj {
    GameAdj::new(lemma)
}

fn adv(lemma: &'static str) -> GameAdv {
    GameAdv::new(lemma)
}

fn verb(lemma: &'static str) -> GameVerb {
    GameVerb::new(lemma)
}

fn pp(prep: &'static str, object: EntityId) -> PrepPhrase {
    PrepPhrase { prep, object }
}

fn action(actor: EntityId, predicate: GameVerb) -> ActionTriple {
    ActionTriple { actor, predicate }
}

impl GameAdj {
    fn new(lemma: &'static str) -> Self {
        Self {
            lemma,
            degree: Degree::Positive,
            complements: Vec::new(),
        }
    }

    fn degree(mut self, degree: Degree) -> Self {
        self.degree = degree;
        self
    }

    fn complement(mut self, prep: &'static str, object: EntityId) -> Self {
        self.complements.push(pp(prep, object));
        self
    }
}

impl GameAdv {
    fn new(lemma: &'static str) -> Self {
        Self {
            lemma,
            modifier: None,
        }
    }

    fn modified_by(mut self, modifier: &'static str) -> Self {
        self.modifier = Some(modifier);
        self
    }
}

impl GameNoun {
    fn common(lemma: &'static str) -> Self {
        Self {
            lemma,
            proper_name: None,
            determiner: Determiner::Definite,
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }

    fn proper(name: &'static str) -> Self {
        Self {
            lemma: name,
            proper_name: Some(name),
            determiner: Determiner::Bare,
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }

    fn determiner(mut self, determiner: Determiner) -> Self {
        self.determiner = determiner;
        self
    }

    fn definite(self) -> Self {
        self.determiner(Determiner::Definite)
    }

    fn modifier(mut self, modifier: GameAdj) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn complement(mut self, prep: &'static str, object: EntityId) -> Self {
        self.complements.push(pp(prep, object));
        self
    }
}

impl GameVerb {
    fn new(lemma: &'static str) -> Self {
        Self {
            lemma,
            modifiers: Vec::new(),
            complements: Vec::new(),
            adjuncts: Vec::new(),
        }
    }

    fn modifier(mut self, modifier: GameAdv) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn object(mut self, object: EntityId) -> Self {
        self.complements.push(VerbComplement::Object(object));
        self
    }

    fn complement(mut self, prep: &'static str, object: EntityId) -> Self {
        self.complements
            .push(VerbComplement::Prep(pp(prep, object)));
        self
    }

    fn adjunct(mut self, prep: &'static str, object: EntityId) -> Self {
        self.adjuncts.push(pp(prep, object));
        self
    }
}

impl Entity {
    fn common(lemma: &'static str, gender: Gender, animacy: Animacy) -> Self {
        Self {
            noun: GameNoun::common(lemma).definite(),
            number: Number::Singular,
            person: Person::Third,
            gender,
            animacy,
        }
    }

    fn proper(name: &'static str, gender: Gender, animacy: Animacy) -> Self {
        Self {
            noun: GameNoun::proper(name),
            number: Number::Singular,
            person: Person::Third,
            gender,
            animacy,
        }
    }

    fn modifier(mut self, modifier: GameAdj) -> Self {
        self.noun = self.noun.modifier(modifier);
        self
    }

    fn complement(mut self, prep: &'static str, object: EntityId) -> Self {
        self.noun = self.noun.complement(prep, object);
        self
    }

    #[allow(dead_code)]
    fn number(mut self, number: Number) -> Self {
        self.number = number;
        self
    }
}

impl Viewpoint {
    fn present() -> Self {
        Self {
            tense: Tense::Present,
            speaker: None,
            listener: None,
        }
    }

    fn past() -> Self {
        Self {
            tense: Tense::Past,
            speaker: None,
            listener: None,
        }
    }

    fn speaker(mut self, speaker: EntityId) -> Self {
        self.speaker = Some(speaker);
        self
    }

    fn listener(mut self, listener: EntityId) -> Self {
        self.listener = Some(listener);
        self
    }
}

impl World {
    fn add(&mut self, entity: Entity) -> EntityId {
        let id = self.entities.len();
        self.entities.push(entity);
        id
    }

    fn simple_sentence(&self, action: &ActionTriple, view: Viewpoint) -> String {
        let subject = self.mention(
            action.actor,
            Case::Nominative,
            view,
            MentionStyle::DeicticOrDefinite,
        );
        let predicate =
            self.action_core(action, view, Form::Finite, MentionStyle::DeicticOrDefinite);
        format!("{} {}.", English::capitalize_first(&subject), predicate)
    }

    fn witness_sentence(
        &self,
        witness: EntityId,
        action: &ActionTriple,
        view: Viewpoint,
    ) -> String {
        let subject = self.mention(
            witness,
            Case::Nominative,
            view,
            MentionStyle::DeicticOrDefinite,
        );
        let (person, number, _) = self.agreement(witness, view);
        let matrix_verb = English::verb("see", &person, &number, &view.tense, &Form::Finite);
        let embedded = self.action_core(action, view, Form::Infinitive, MentionStyle::Introduce);
        format!(
            "{} {} {}.",
            English::capitalize_first(&subject),
            matrix_verb,
            embedded
        )
    }

    fn action_core(
        &self,
        action: &ActionTriple,
        view: Viewpoint,
        form: Form,
        complement_mentions: MentionStyle,
    ) -> String {
        let mut parts = Vec::new();

        if matches!(form, Form::Infinitive) {
            parts.push(self.mention(action.actor, Case::Nominative, view, complement_mentions));
        }

        parts.extend(action.predicate.modifiers.iter().map(render_adv));

        let (person, number, _) = self.agreement(action.actor, view);
        parts.push(English::verb(
            action.predicate.lemma,
            &person,
            &number,
            &view.tense,
            &form,
        ));

        for complement in &action.predicate.complements {
            let text = match complement {
                VerbComplement::Object(object) => {
                    self.mention(*object, Case::Accusative, view, complement_mentions)
                }
                VerbComplement::Prep(phrase) => {
                    self.prep_phrase(*phrase, view, complement_mentions)
                }
            };
            parts.push(text);
        }

        for adjunct in &action.predicate.adjuncts {
            parts.push(self.prep_phrase(*adjunct, view, MentionStyle::DeicticOrDefinite));
        }

        parts.join(" ")
    }

    fn mention(
        &self,
        entity_id: EntityId,
        case: Case,
        view: Viewpoint,
        mention: MentionStyle,
    ) -> String {
        let entity = &self.entities[entity_id];

        if matches!(mention, MentionStyle::DeicticOrDefinite)
            && (view.speaker == Some(entity_id) || view.listener == Some(entity_id))
        {
            let (person, number, gender) = self.agreement(entity_id, view);
            return English::pronoun(&person, &number, &gender, &case).to_string();
        }

        if let Some(name) = entity.noun.proper_name {
            return name.to_string();
        }

        let mention_noun = match mention {
            MentionStyle::DeicticOrDefinite => entity.noun.clone(),
            MentionStyle::Introduce => entity.noun.clone().determiner(Determiner::Indefinite),
        };

        let mut words = Vec::new();
        if let Some(det) = render_determiner(&mention_noun) {
            words.push(det);
        }

        let (prenominal, postnominal): (Vec<_>, Vec<_>) = mention_noun
            .modifiers
            .iter()
            .partition(|modifier| modifier.complements.is_empty());

        words.extend(
            prenominal
                .into_iter()
                .map(|modifier| self.adjective(modifier, view)),
        );
        words.push(English::noun(mention_noun.lemma, &entity.number));
        words.extend(
            postnominal
                .into_iter()
                .map(|modifier| self.adjective(modifier, view)),
        );
        words.extend(
            mention_noun
                .complements
                .into_iter()
                .map(|phrase| self.prep_phrase(phrase, view, MentionStyle::DeicticOrDefinite)),
        );

        words.join(" ")
    }

    fn adjective(&self, adjective: &GameAdj, view: Viewpoint) -> String {
        let mut text = English::adj(adjective.lemma, &adjective.degree);
        if !adjective.complements.is_empty() {
            let complements = adjective
                .complements
                .iter()
                .map(|phrase| self.prep_phrase(*phrase, view, MentionStyle::DeicticOrDefinite))
                .collect::<Vec<_>>()
                .join(" ");
            text.push(' ');
            text.push_str(&complements);
        }
        text
    }

    fn prep_phrase(&self, phrase: PrepPhrase, view: Viewpoint, mention: MentionStyle) -> String {
        format!(
            "{} {}",
            phrase.prep,
            self.mention(phrase.object, Case::Accusative, view, mention)
        )
    }

    fn agreement(&self, entity_id: EntityId, view: Viewpoint) -> (Person, Number, Gender) {
        let entity = &self.entities[entity_id];
        let person = if view.speaker == Some(entity_id) {
            Person::First
        } else if view.listener == Some(entity_id) {
            Person::Second
        } else {
            entity.person
        };

        let gender = if matches!(person, Person::First | Person::Second)
            || matches!(entity.animacy, Animacy::Inanimate)
        {
            Gender::Neuter
        } else {
            entity.gender
        };

        (person, entity.number, gender)
    }
}

fn render_adv(adverb: &GameAdv) -> String {
    match adverb.modifier {
        Some(modifier) => format!("{modifier} {}", adverb.lemma),
        None => adverb.lemma.to_string(),
    }
}

fn render_determiner(noun: &GameNoun) -> Option<String> {
    match noun.determiner {
        Determiner::Definite => Some("the".to_string()),
        Determiner::Indefinite => Some(indefinite_article(next_surface_word(noun)).to_string()),
        Determiner::Bare => None,
    }
}

fn next_surface_word(noun: &GameNoun) -> &str {
    noun.modifiers
        .first()
        .map(|modifier| modifier.lemma)
        .unwrap_or(noun.lemma)
}

fn indefinite_article(next_word: &str) -> &'static str {
    match next_word.chars().next().map(|ch| ch.to_ascii_lowercase()) {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}
