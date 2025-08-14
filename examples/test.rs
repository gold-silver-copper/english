use english::*;

fn main() {
    test_all_verb_conjugations();

    // Decline a noun (handles irregulars)
    let plural = English::noun("child", &Number::Plural);
    println!("child (plural) -> {}", plural); // children
    let plural = English::noun("die", &Number::Plural);
    println!("die (plural) -> {}", plural); // children
    let plural = English::noun("die2", &Number::Plural);
    println!("die2 (plural) -> {}", plural); // children

    // Regular forms
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
}

fn test_all_verb_conjugations() {
    let verbs = vec!["walk", "run", "be", "eat", "lie", "lie2"];

    let persons = vec![Person::First, Person::Second, Person::Third];
    let numbers = vec![Number::Singular, Number::Plural];
    let tenses = vec![Tense::Present, Tense::Past];
    let forms = vec![Form::Finite, Form::Participle, Form::Infinitive];

    for verb_word in verbs {
        println!("Testing verb: {}", verb_word);
        for person in &persons {
            for number in &numbers {
                for tense in &tenses {
                    for form in &forms {
                        let result = English::verb(verb_word, person, number, tense, form);
                        println!(
                            "{:?} {:?} {:?} {:?} -> {}",
                            person, number, tense, form, result
                        );
                    }
                }
            }
        }
    }
}
