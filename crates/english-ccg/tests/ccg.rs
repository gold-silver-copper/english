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

    let s = name(alice.clone()) + verb(arrive.clone()).past();
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "Alice arrived.");

    let s = name(jordan.clone()) + verb(trust.clone()).past() + name(alice.clone());
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Jordan trusted Alice."
    );

    let s = name(alice.clone())
        + verb(arrive.clone()).past()
        + prep("before").adverbial()
        + (det("the") + noun(inspection.clone()));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice arrived before the inspection."
    );

    let s = name(alice.clone())
        + verb(inspect.clone()).past()
        + (det("the")
            + (noun(bridge.clone()) + prep("of").adnominal() + (det("the") + noun(city.clone()))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the bridge of the city."
    );

    let pp = prep("before") + (det("the") + noun(inspection.clone()));
    assert_eq!(pp.cat().to_notation(), "PP");

    let s = name(alice.clone())
        + verb(inspect.clone()).past()
        + (det("the") + (verb(damage.clone()).past_participle() + noun(bridge.clone())));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the damaged bridge."
    );

    let s = name(alice.clone())
        + verb(inspect.clone()).past()
        + (det("the") + (verb(run.clone()).present_participle() + noun(water.clone())));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice inspected the running water."
    );

    let s = name(alice.clone())
        + modal(Modal::Have)
        + verb(repair.clone()).perfective()
        + (det("the") + noun(bridge.clone()));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice has repaired the bridge."
    );

    let s = name(alice.clone())
        + modal(Modal::Be)
        + verb(repair.clone()).progressive()
        + (det("the") + noun(bridge.clone()));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice is repairing the bridge."
    );

    let s = (det("the") + noun(bridge.clone())) + verb(repair.clone()).passive();
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "The bridge was repaired."
    );

    let s = (det("the") + noun(bridge.clone()))
        + verb(repair.clone()).passive_by()
        + name(alice.clone());
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "The bridge was repaired by Alice."
    );

    let s = name(jordan.clone()) + verb(trust.clone()).present() + name(alice.clone());
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Jordan trusts Alice."
    );

    let s = pronoun(Pronoun::She) + verb(trust.clone()).past() + pronoun(Pronoun::Him);
    assert_eq!(realize_as(&s, RealizeOpts::sentence()), "She trusted him.");

    let s = name(alice.clone())
        + verb(trust.clone()).past()
        + (det("the")
            + (noun(engineer.clone())
                + (rel("who")
                    + (name(jordan.clone()) + verb(trust.clone()).past() + gap(cat!("NP"))))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice trusted the engineer who Jordan trusted."
    );

    let s =
        coord(Conj::And, name(alice.clone()), name(jordan.clone())) + verb(arrive.clone()).past();
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Alice and Jordan arrived."
    );

    let s = name(jordan.clone())
        + verb(say.clone()).past()
        + comp("that")
        + ((det("the")
            + adj("smart")
            + (noun(engineer.clone())
                + (rel("who")
                    + (name(alice.clone()) + verb(trust.clone()).past() + gap(cat!("NP"))))))
            + verb(promise.clone()).past()
            + inf()
            + verb(repair.clone()).bare()
            + (det("the") + (verb(damage.clone()).past_participle() + noun(bridge.clone())))
            + prep("before").adverbial()
            + (det("the") + (adj("final") + noun(inspection.clone()))));
    assert_eq!(
        realize_as(&s, RealizeOpts::sentence()),
        "Jordan said that the smart engineer who Alice trusted promised to repair the damaged bridge before the final inspection."
    );
}
