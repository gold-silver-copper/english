use english::Animacy;
use english_ccg::cat;
use english_ccg::prelude::*;

#[test]
fn entry_driven_ccg_examples() {
    let alice = proper("Alice");
    let jordan = proper("Jordan");
    let arrive = iv("arrive");
    let trust = tv("trust");
    let inspect = tv("inspect");
    let repair = tv("repair");
    let damage = tv("damage");
    let run = iv("run");
    let say = scomp("say");
    let promise = vpcomp("promise");

    let inspection = common("inspection", Animacy::Inanimate);
    let bridge = common("bridge", Animacy::Inanimate);
    let city = common("city", Animacy::Inanimate);
    let engineer = common("engineer", Animacy::Animate);
    let water = common("water", Animacy::Inanimate);

    let s = name(&alice) + verb(&arrive).past();
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "Alice arrived.");

    let s = name(&jordan) + verb(&trust).past() + name(&alice);
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Jordan trusted Alice."
    );

    let s = name(&alice)
        + verb(&arrive).past()
        + prep("before").adverbial()
        + (det("the") + noun(&inspection));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice arrived before the inspection."
    );

    let s = name(&alice)
        + verb(&inspect).past()
        + (det("the") + (noun(&bridge) + prep("of").adnominal() + (det("the") + noun(&city))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the bridge of the city."
    );

    let pp = prep("before") + (det("the") + noun(&inspection));
    assert_eq!(pp.cat().to_notation(), "PP");

    let s = name(&alice)
        + verb(&inspect).past()
        + (det("the") + (verb(&damage).past_participle() + noun(&bridge)));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the damaged bridge."
    );

    let s = name(&alice)
        + verb(&inspect).past()
        + (det("the") + (verb(&run).present_participle() + noun(&water)));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the running water."
    );

    let s = name(&alice)
        + modal(Modal::Have)
        + verb(&repair).perfective()
        + (det("the") + noun(&bridge));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice has repaired the bridge."
    );

    let s = name(&alice)
        + modal(Modal::Be)
        + verb(&repair).progressive()
        + (det("the") + noun(&bridge));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice is repairing the bridge."
    );

    let s = (det("the") + noun(&bridge)) + verb(&repair).passive();
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "The bridge was repaired."
    );

    let s = (det("the") + noun(&bridge)) + verb(&repair).passive_by() + name(&alice);
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "The bridge was repaired by Alice."
    );

    let s = name(&jordan) + verb(&trust).present() + name(&alice);
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Jordan trusts Alice."
    );

    let s = pronoun(Pronoun::She) + verb(&trust).past() + pronoun(Pronoun::Him);
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "She trusted him.");

    let s = name(&alice)
        + verb(&trust).past()
        + (det("the")
            + (noun(&engineer)
                + (rel("who") + (name(&jordan) + verb(&trust).past() + gap(cat!(r"NP"))))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice trusted the engineer who Jordan trusted."
    );

    let s = coord(Conj::And, name(&alice), name(&jordan)) + verb(&arrive).past();
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice and Jordan arrived."
    );

    let s = name(&jordan)
        + verb(&say).past()
        + comp("that")
        + ((det("the")
            + adj("smart")
            + (noun(&engineer)
                + (rel("who") + (name(&alice) + verb(&trust).past() + gap(cat!(r"NP"))))))
            + verb(&promise).past()
            + inf()
            + verb(&repair).bare()
            + (det("the") + (verb(&damage).past_participle() + noun(&bridge)))
            + prep("before").adverbial()
            + (det("the") + (adj("final") + noun(&inspection))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Jordan said that the smart engineer who Alice trusted promised to repair the damaged bridge before the final inspection."
    );
}
