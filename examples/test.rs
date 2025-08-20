use english::*;
fn main() {
    // --- Mixed Sentence Example ---
    let subject_number = Number::Plural;
    let run = English::present_participle("run"); // running
    let child = Noun::from("child").with_specifier(run); //running child
    let subject = English::noun(child, &subject_number); //running children
    let verb = English::verb(
        "steal",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    ); //stole
    let object = English::count_with_number("potato", 7); //7 potatoes

    let sentence = format!("The {} {} {}.", subject, verb, object);
    assert_eq!(sentence, "The running children stole 7 potatoes.");

    // --- Nouns ---
    // Note that noun(), count(), etc can work on both strings and Noun struct
    let jeans = Noun::from("pair").with_complement("of jeans");
    assert_eq!(English::count_with_number(jeans, 3), "3 pairs of jeans");
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    // Add a number 2-9 to the end of the word to try different forms.
    // Can use plural()
    assert_eq!(English::plural("die2"), "dice");
    // Use count function for better ergonomics if needed
    assert_eq!(English::count("man", 2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(English::count_with_number("nickel", 3), "3 nickels");
    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // --- Verbs ---
    // All verb functions can use either strings or Verb struct
    let pick_up = Verb::from("pick").with_particle("up");
    // Helper functions: past() , third_person(), present_participle(), infinitive() etc.
    assert_eq!(English::past(&pick_up,), "picked up");
    assert_eq!(English::present_participle("walk"), "walking");
    assert_eq!(English::past_participle("go"), "gone");
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(English::past("lie"), "lay");
    assert_eq!(English::past("lie2"), "lied");
    // "to be" has the most verb forms in english and requires using verb()
    assert_eq!(
        English::verb(
            "be",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite
        ),
        "am"
    );

    // --- Adjectives ---
    // Add a number 2-9 to the end of the word to try different forms. (Bad has the most forms at 3)
    assert_eq!(English::adj("bad", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("bad", &Degree::Superlative), "most bad");
    assert_eq!(English::comparative("bad2"), "badder");
    assert_eq!(English::superlative("bad2"), "baddest");
    assert_eq!(English::comparative("bad3"), "worse");
    assert_eq!(English::superlative("bad3"), "worst");
    assert_eq!(English::positive("bad3"), "bad");

    // --- Pronouns ---
    assert_eq!(
        English::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Neuter,
            &Case::PersonalPossesive
        ),
        "my"
    );
    assert_eq!(
        English::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Neuter,
            &Case::Possessive
        ),
        "mine"
    );

    // --- Possessives ---
    assert_eq!(English::add_possessive("dog"), "dog's");
    assert_eq!(English::add_possessive("dogs"), "dogs'");
}
