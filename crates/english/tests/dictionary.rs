//! Tests for the optional `dictionary` feature (`English::*_meanings`).
//! The whole file is gated, so it is a no-op when the feature is off.
#![cfg(feature = "dictionary")]

use english::English;

fn any_contains(defs: &[&str], needle: &str) -> bool {
    defs.iter().any(|d| d.to_lowercase().contains(needle))
}

#[test]
fn definitions_are_keyed_by_sense() {
    // die2 is the cube/dice sense.
    let die2 = English::noun_meanings("die2");
    assert!(!die2.is_empty(), "die2 should have definitions");
    assert!(
        any_contains(die2, "cube") || any_contains(die2, "polyhedron") || any_contains(die2, "dice"),
        "die2 defs unexpected: {die2:?}"
    );

    // The two `lie` homographs must carry DIFFERENT definitions.
    let lie = English::verb_meanings("lie");
    let lie2 = English::verb_meanings("lie2");
    assert!(!lie.is_empty() && !lie2.is_empty());
    assert_ne!(lie, lie2, "recline vs untruth must differ");
    assert!(any_contains(lie2, "false") || any_contains(lie2, "deceive"));
}

#[test]
fn unknown_or_regular_keys_return_empty() {
    assert!(English::verb_meanings("definitelynotaword").is_empty());
    assert!(English::noun_meanings("definitelynotaword").is_empty());
    assert!(English::adj_meanings("definitelynotaword").is_empty());
}
