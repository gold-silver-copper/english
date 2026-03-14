use english::*;
fn main() {
    // --- Mixed Sentence Example ---
    let subject_number = Number::Plural;
    let subject = format!(
        "{} {}",
        Verb::new("run").present_participle(),
        English::noun("child", &subject_number)
    ); // running children
    let verb = English::verb(
        "steal",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    ); //stole
    let object = Noun::new("potato").count_with_number(7); //7 potatoes

    let sentence = format!("The {} {} {}.", subject, verb, object);
    assert_eq!(sentence, "The running children stole 7 potatoes.");

    // --- Nouns ---
    assert_eq!(
        format!("{} of jeans", Noun::new("pair").count_with_number(3)),
        "3 pairs of jeans"
    );
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    // Add a number 2-9 to the end of the word to try different forms.
    // Can use plural()
    assert_eq!(Noun::new("die2").plural(), "dice");
    // Use count function for better ergonomics if needed
    assert_eq!(Noun::new("man").count(2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(Noun::new("nickel").count_with_number(3), "3 nickels");
    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // --- Verbs ---
    // Verb functions operate on the base lemma only.
    // Helper functions: past() , third_person(), present_participle(), infinitive() etc.
    assert_eq!(Verb::new("pick").past(), "picked");
    assert_eq!(Verb::new("walk").present_participle(), "walking");
    assert_eq!(Verb::new("go").past_participle(), "gone");
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(Verb::new("lie").past(), "lay");
    assert_eq!(Verb::new("lie2").past(), "lied");
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
    assert_eq!(Adj::new("bad2").comparative(), "badder");
    assert_eq!(Adj::new("bad2").superlative(), "baddest");
    assert_eq!(Adj::new("bad3").comparative(), "worse");
    assert_eq!(Adj::new("bad3").superlative(), "worst");
    assert_eq!(Adj::new("bad3").positive(), "bad");

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
