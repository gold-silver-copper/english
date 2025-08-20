use english::*;
fn main() {
    assert_eq!(English::third_person("run"), "runs");
    assert_eq!(English::past("walk"), "walked");
    assert_eq!(English::present_participle("swim"), "swimming");
    assert_eq!(English::past_participle("eat"), "eaten");
    assert_eq!(English::infinitive("go"), "go");
    assert_eq!(English::plural("child"), "children");
    assert_eq!(English::plural("cat"), "cats");
    assert_eq!(English::singular("cat2"), "cat");
    assert_eq!(English::comparative("fast2"), "faster");
    assert_eq!(English::comparative("fun"), "more fun");
    assert_eq!(English::superlative("fast2"), "fastest");
    assert_eq!(English::positive("fast2"), "fast");
    assert_eq!(English::superlative("fun"), "most fun");
    assert_eq!(English::capitalize_first(""), "");
    assert_eq!(English::capitalize_first("house"), "House");
}
