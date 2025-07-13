use english::*;

fn main() {
    // Conjugate a verb (handles irregulars)
    let past = English::verb(
        "eat",
        &Person::Third,
        &Number::Singular,
        &Tense::Past,
        &Form::Finite,
    );
    println!("eat (past) -> {}", past); // ate

    // Decline a noun (handles irregulars)
    let plural = English::noun("child", &Number::Plural);
    println!("child (plural) -> {}", plural); // children

    // Regular forms
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
}
