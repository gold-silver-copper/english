use english::*;

#[test]
fn base_surface_helpers_work() {
    assert_eq!(Verb::new("run").third_person(), "runs");
    assert_eq!(Verb::new("walk").past(), "walked");
    assert_eq!(Verb::new("swim").present_participle(), "swimming");
    assert_eq!(Verb::new("eat").past_participle(), "eaten");
    assert_eq!(Verb::new("go").infinitive(), "go");
    assert_eq!(Noun::new("child").plural(), "children");
    assert_eq!(Noun::new("cat").plural(), "cats");
    assert_eq!(Noun::new("cat2").singular(), "cat");
    assert_eq!(English::adj("bad", &Degree::Comparative), "more bad");
    assert_eq!(Adj::new("fun").comparative(), "more fun");
    assert_eq!(Adj::new("bad2").comparative(), "badder");
    assert_eq!(Adj::new("bad3").positive(), "bad");
    assert_eq!(Adj::new("fun").superlative(), "most fun");
    assert_eq!(Adj::new("bad3").superlative(), "worst");
    assert_eq!(English::capitalize_first(""), "");
    assert_eq!(English::capitalize_first("house"), "House");
}
