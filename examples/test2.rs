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
}
