use english::*;
fn main() {
    // --- Nouns ---
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");

    // Irregular plurals
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(English::noun("child", &Number::Plural), "children");
    assert_eq!(English::noun("die", &Number::Plural), "dies");
    assert_eq!(English::noun("die2", &Number::Plural), "dice");

    // Use count function for better ergonomics if needed
    assert_eq!(English::count("man", 2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(English::count_with_number("nickel", 3), "3 nickels");

    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // Complex nouns, note that From<&str> is impl'd for Noun
    // Note that noun(), count(), etc can work on both strings and Noun struct
    let jeans = Noun::from("pair").with_complement("of jeans");
    assert_eq!(English::count_with_number(jeans, 3), "3 pairs of jeans");

    // --- Adjectives ---
    // Regular adjectives
    assert_eq!(English::adj("fast", &Degree::Comparative), "faster");

    // Irregular adjectives
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(English::adj("bad", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("bad", &Degree::Superlative), "most bad");
    assert_eq!(English::adj("bad2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad2", &Degree::Superlative), "baddest");
    assert_eq!(English::adj("bad3", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad3", &Degree::Superlative), "worst");

    // Complex verbs, note that From<&str> is impl'd for Verb
    let pick_up = Verb::from("pick").with_particle("up");

    // Past tense, third person singular
    assert_eq!(
        English::verb(
            &pick_up,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "picked up"
    );

    // --- Verbs ---
    // Regular verbs, note that verb() can be used on both strings and Verb struct
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

    // Irregular verbs
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
    assert_eq!(
        English::verb(
            "go",
            &Person::Third,
            &Number::Plural,
            &Tense::Past,
            &Form::Participle
        ),
        "gone"
    );
    assert_eq!(
        English::verb(
            "lie",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lay"
    );
    assert_eq!(
        English::verb(
            "lie2",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lied"
    );

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

    // --- Mixed Sentence Example ---
    let subject_number = Number::Plural;
    let subject = English::noun("child", &subject_number);
    let verb = English::verb(
        "play",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    );
    let object = English::noun("die2", &Number::Plural);

    let sentence = format!("The {} {} with {}.", subject, verb, object);
    assert_eq!(sentence, "The children played with dice.");
}
