use english::*;

pub struct Entity {
    id: EntityID,
    location: Locative,
}
pub type EntityID = usize;
pub type Subject = EntityID;
pub type Object = EntityID;

pub enum ActionType {
    Walk(Subject, Direction),
    Eat(Subject, Object),
    Say(Subject, Utterance),
}
pub enum Direction {
    North,
    East,
    South,
    West,
}
pub enum Connection {
    Door,
    Road,
}
pub struct GridPoint {
    x: isize,
    y: isize,
}
pub type PlaceID = usize;
pub type ConnectionID = usize;

pub enum Locative {
    InEntity(EntityID),
    Point(GridPoint),
}
pub enum Consumable {
    Food,
    Drink,
}
pub enum Utterance {
    Sentence,
    Grunt,
}

fn main() {
    println!(
        "{:#?}",
        EnglishCore::verb(
            "eat",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        )
    );

    let noun1 = Noun {
        word: "chicanery".to_string(),
        number: Number::Plural,
    };
    let verb1 = Verb {
        word: "eat".to_string(),
        tense: Tense::Past,
        person: Person::Third,
        form: Form::Finite,
    };
    println!("{:#?}", English::simple_sentence(&noun1, &noun1, &verb1));

    println!("{}", English::noun("thyridium", &Number::Plural));
}
