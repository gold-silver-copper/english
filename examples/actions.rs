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

pub struct Human {
    name: String,
    eye_color: EyeColor,
    hair_color: HairColor,
    skin_color: SkinColor,
    height: usize,
    weight: usize,
    age: usize,
    gender: Gender,
    location: Locative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HairStyle {
    Bald,
    Short,
    Medium,
    Long,
    Ponytail,
    Bun,
    Braided,
    Curly,
    Wavy,
    Straight,
    Tangled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacialHair {
    Mustache,
    Beard,
    Stubble,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EyeShape {
    Almond,
    Round,
    Hooded,
    Monolid,
    Downturned,
    Upturned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoseType {
    Button,
    Roman,
    Straight,
    Nubian,
    Hawk,
    Snub,
    Aquiline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EarType {
    Elven,
    Wolf,
    Dog,
    Cat,
    Human,
    Fox,
}
pub struct Ears {
    ear_type: EarType,
    ear_size: BasicSize,
}

pub enum BasicSize {
    Small,
    Medium,
    Big,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Freckles {
    Few,
    Moderate,
    Many,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tattoo {
    Geometric,
    Animal(AnimalType),
    Script(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimalType {
    Wolf,
    Fox,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HairColor {
    Blonde,
    Brunette,
    Black,
    Ginger,
    Gray,
    White,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkinColor {
    Pale,
    Fair,
    Olive,
    Sunkissed,
    Tan,
}

pub enum EyeColor {
    Blue,
    Green,
    Gray,
    Hazel,
    Brown,
    Amber,
    Violet,
    Red,
}

/*#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scars {
    None,
    Small,
    Noticeable,
    Prominent,
    AcrossEye,
    Burn,
} */
