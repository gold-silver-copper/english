use english::*;
fn main() {
    // --- Mixed Sentence Example ---
    let subject_number = Number::Plural;
    let subject = format!(
        "{} {}",
        English::verb(
            "run",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle
        ),
        English::noun("child", &subject_number)
    ); // running children
    let verb = English::verb(
        "steal",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    ); //stole
    let object = count_with_number("potato", 7); //7 potatoes

    let sentence = format!("The {} {} {}.", subject, verb, object);
    assert_eq!(sentence, "The running children stole 7 potatoes.");

    // --- Nouns ---
    assert_eq!(
        format!("{} of jeans", count_with_number("pair", 3)),
        "3 pairs of jeans"
    );
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(English::noun("die2", &Number::Plural), "dice");
    // Use count function for better ergonomics if needed
    assert_eq!(count("man", 2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(count_with_number("nickel", 3), "3 nickels");
    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // --- Verbs ---
    assert_eq!(
        English::verb(
            "pick",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "picked"
    );
    assert_eq!(
        English::verb(
            "walk",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle
        ),
        "walking"
    );
    assert_eq!(
        English::verb(
            "go",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Participle
        ),
        "gone"
    );
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(
        English::verb(
            "lie",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lay"
    );
    assert_eq!(
        English::verb(
            "lie2",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lied"
    );
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
    assert_eq!(English::adj("bad2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad2", &Degree::Superlative), "baddest");
    assert_eq!(English::adj("bad3", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad3", &Degree::Superlative), "worst");
    assert_eq!(English::adj("bad3", &Degree::Positive), "bad");

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
