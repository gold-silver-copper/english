use english::*;
fn main() {
    assert_eq!(Verb::third_person("run"), "runs");
    assert_eq!(Verb::past("walk"), "walked");
    assert_eq!(Verb::present_participle("swim"), "swimming");
    assert_eq!(Verb::past_participle("eat"), "eaten");
    assert_eq!(Verb::infinitive("go"), "go");
    assert_eq!(Noun::plural("child"), "children");
    assert_eq!(Noun::plural("cat"), "cats");
    assert_eq!(Noun::singular("cat2"), "cat");
    assert_eq!(Adj::comparative("fast2"), "faster");
    assert_eq!(Adj::comparative("fun"), "more fun");
    assert_eq!(Adj::superlative("fast2"), "fastest");
    assert_eq!(Adj::positive("fast2"), "fast");
    assert_eq!(Adj::superlative("fun"), "most fun");
    assert_eq!(English::capitalize_first(""), "");
    assert_eq!(English::capitalize_first("house"), "House");
    let pick_up = Verb::from("pick").with_particle("up");
    assert_eq!(Verb::past_participle(pick_up), "picked up");

    // Simple forms
    assert_eq!(Verb::negate("eat"), "not eat");
    assert_eq!(Verb::future("eat"), "will eat");
    assert_eq!(Verb::emphatic_past("eat"), "did eat");
    assert_eq!(Verb::conditional("eat"), "would eat");
    assert_eq!(Verb::could("eat"), "could eat");
    assert_eq!(Verb::can("eat"), "can eat");
    assert_eq!(Verb::should("eat"), "should eat");

    // Perfect aspects
    assert_eq!(
        Verb::present_perfect("eat", &Person::Third, &Number::Singular),
        "has eaten"
    );
    assert_eq!(
        Verb::present_perfect("eat", &Person::First, &Number::Plural),
        "have eaten"
    );
    assert_eq!(Verb::past_perfect("eat"), "had eaten");
    assert_eq!(Verb::future_perfect("eat"), "will have eaten");

    // Progressive aspects
    assert_eq!(
        Verb::present_progressive("eat", &Person::Third, &Number::Singular),
        "is eating"
    );
    assert_eq!(
        Verb::present_progressive("eat", &Person::First, &Number::Plural),
        "are eating"
    );
    assert_eq!(
        Verb::past_progressive("eat", &Person::Third, &Number::Singular),
        "was eating"
    );
    assert_eq!(
        Verb::past_progressive("eat", &Person::First, &Number::Plural),
        "were eating"
    );
    assert_eq!(Verb::future_progressive("eat"), "will be eating");

    // Negation / modal / emphatic
    assert_eq!(Verb::negate("eat"), "not eat");
    assert_eq!(Verb::negate("see"), "not see");
    assert_eq!(Verb::future("run"), "will run");
    assert_eq!(Verb::emphatic_past("go"), "did go");
    assert_eq!(Verb::conditional("eat"), "would eat");
    assert_eq!(Verb::could("see"), "could see");
    assert_eq!(Verb::can("run"), "can run");
    assert_eq!(Verb::should("go"), "should go");

    // Perfect aspects
    assert_eq!(
        Verb::present_perfect("eat", &Person::Third, &Number::Singular),
        "has eaten"
    );
    assert_eq!(
        Verb::present_perfect("eat", &Person::First, &Number::Plural),
        "have eaten"
    );
    assert_eq!(
        Verb::present_perfect("see", &Person::Third, &Number::Singular),
        "has seen"
    );
    assert_eq!(
        Verb::present_perfect("see", &Person::First, &Number::Plural),
        "have seen"
    );
    assert_eq!(Verb::past_perfect("run"), "had run");
    assert_eq!(Verb::past_perfect("go"), "had gone");
    assert_eq!(Verb::future_perfect("eat"), "will have eaten");
    assert_eq!(Verb::future_perfect("see"), "will have seen");

    // Progressive aspects
    assert_eq!(
        Verb::present_progressive("eat", &Person::Third, &Number::Singular),
        "is eating"
    );
    assert_eq!(
        Verb::present_progressive("eat", &Person::First, &Number::Plural),
        "are eating"
    );
    assert_eq!(
        Verb::present_progressive("run", &Person::Third, &Number::Singular),
        "is running"
    );
    assert_eq!(
        Verb::present_progressive("run", &Person::First, &Number::Plural),
        "are running"
    );
    assert_eq!(
        Verb::past_progressive("eat", &Person::Third, &Number::Singular),
        "was eating"
    );
    assert_eq!(
        Verb::past_progressive("eat", &Person::First, &Number::Plural),
        "were eating"
    );
    assert_eq!(
        Verb::past_progressive("run", &Person::Third, &Number::Singular),
        "was running"
    );
    assert_eq!(
        Verb::past_progressive("run", &Person::First, &Number::Plural),
        "were running"
    );
    assert_eq!(Verb::future_progressive("go"), "will be going");

    // Edge cases: be + have
    assert_eq!(
        Verb::present_perfect("be", &Person::Third, &Number::Singular),
        "has been"
    );
    assert_eq!(
        Verb::present_perfect("be", &Person::First, &Number::Plural),
        "have been"
    );
    assert_eq!(Verb::past_perfect("be"), "had been");
    assert_eq!(Verb::future_perfect("be"), "will have been");

    assert_eq!(
        Verb::present_progressive("have", &Person::Third, &Number::Singular),
        "is having"
    );
    assert_eq!(
        Verb::present_progressive("have", &Person::First, &Number::Plural),
        "are having"
    );
    assert_eq!(
        Verb::past_progressive("have", &Person::Third, &Number::Singular),
        "was having"
    );
    assert_eq!(
        Verb::past_progressive("have", &Person::First, &Number::Plural),
        "were having"
    );
    assert_eq!(Verb::future_progressive("have"), "will be having");

    let give_up = Verb::from("give").with_particle("up");
    assert_eq!(
        Verb::present_perfect(&give_up, &Person::First, &Number::Singular),
        "have given up"
    );
    assert_eq!(
        Verb::present_perfect(give_up, &Person::Third, &Number::Singular),
        "has given up"
    );
    // Complex phrasal verb with aspect
    let look_up = Verb::from("look").with_particle("up");
    assert_eq!(
        Verb::past_progressive(&look_up, &Person::Third, &Number::Singular),
        "was looking up"
    );

    assert_eq!(
        Verb::past_progressive(look_up, &Person::Third, &Number::Plural),
        "were looking up"
    );
}
