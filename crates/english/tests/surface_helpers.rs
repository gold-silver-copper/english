use english::*;

#[test]
fn base_surface_helpers_work() {
    assert_eq!(
        English::verb(
            "run",
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite
        ),
        "runs"
    );
    assert_eq!(
        English::verb(
            "walk",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "walked"
    );
    assert_eq!(
        English::verb(
            "swim",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle
        ),
        "swimming"
    );
    assert_eq!(
        English::verb(
            "eat",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Participle
        ),
        "eaten"
    );
    assert_eq!(
        English::verb(
            "go",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Infinitive
        ),
        "go"
    );
    assert_eq!(English::noun("child", &Number::Plural), "children");
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    assert_eq!(English::noun("cat2", &Number::Singular), "cat");
    assert_eq!(count("man", 2), "men");
    assert_eq!(count_with_number("nickel", 3), "3 nickels");
    assert_eq!(English::adj("bad", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("fun", &Degree::Comparative), "more fun");
    assert_eq!(English::adj("bad2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad3", &Degree::Positive), "bad");
    assert_eq!(English::adj("fun", &Degree::Superlative), "most fun");
    assert_eq!(English::adj("bad3", &Degree::Superlative), "worst");
    assert_eq!(English::capitalize_first(""), "");
    assert_eq!(English::capitalize_first("house"), "House");
}
