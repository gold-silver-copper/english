use english_ccg::cat;
use english_ccg::prelude::*;

#[test]
fn required_examples() {
    let s = name("Alice") + verb("arrive").past();
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "Alice arrived.");

    let s = name("Jordan") + verb("trust").past() + name("Alice");
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "Jordan trusted Alice.");

    let s = name("Alice")
        + verb("arrive").past()
        + prep("before").adverbial()
        + (det("the") + noun("inspection"));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice arrived before the inspection."
    );

    let s = name("Alice")
        + verb("inspect").past()
        + (det("the") + (noun("bridge") + prep("of").adnominal() + (det("the") + noun("city"))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the bridge of the city."
    );

    let pp = prep("before") + (det("the") + noun("inspection"));
    assert_eq!(pp.cat().to_notation(), "PP");

    let s = name("Alice")
        + verb("inspect").past()
        + (det("the") + (verb("damage").past_participle() + noun("bridge")));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the damaged bridge."
    );

    let s = name("Alice")
        + verb("inspect").past()
        + (det("the") + (verb("run").present_participle() + noun("water")));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the running water."
    );

    let s = name("Alice")
        + modal(Modal::Have)
        + verb("repair").perfective()
        + (det("the") + noun("bridge"));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice has repaired the bridge."
    );

    let s = name("Alice")
        + modal(Modal::Be)
        + verb("repair").progressive()
        + (det("the") + noun("bridge"));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice is repairing the bridge."
    );

    let s = (det("the") + noun("bridge")) + verb("repair").passive();
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "The bridge was repaired.");

    let s = (det("the") + noun("bridge")) + verb("repair").passive_by() + name("Alice");
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "The bridge was repaired by Alice."
    );

    let s = name("Jordan") + verb("trust").present() + name("Alice");
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "Jordan trusts Alice.");

    let s = pronoun(Pronoun::She) + verb("trust").past() + pronoun(Pronoun::Him);
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "She trusted him.");

    let s = name("Alice")
        + verb("trust").past()
        + (det("the")
            + (noun("engineer")
                + (rel("who") + (name("Jordan") + verb("trust").past() + gap(cat!("NP"))))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice trusted the engineer who Jordan trusted."
    );

    let s = coord(Conj::And, name("Alice"), name("Jordan")) + verb("arrive").past();
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice and Jordan arrived."
    );

    let s = name("Jordan")
        + verb("say").past()
        + comp("that")
        + ((det("the")
            + adj("smart")
            + (noun("engineer")
                + (rel("who") + (name("Alice") + verb("trust").past() + gap(cat!("NP"))))))
            + verb("promise").past()
            + inf()
            + verb("repair").bare()
            + (det("the") + (verb("damage").past_participle() + noun("bridge")))
            + prep("before").adverbial()
            + (det("the") + (adj("final") + noun("inspection"))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Jordan said that the smart engineer who Alice trusted promised to repair the damaged bridge before the final inspection."
    );
}
