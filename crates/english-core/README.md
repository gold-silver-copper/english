# english-core

The dependency-free **regular fallback / prediction** engine underneath the
[`english`](https://crates.io/crates/english) crate.

`english-core` knows nothing about Wiktionary or lookup tables. It encodes a
small approximation of productive English morphology — add `-s`, `-ed`,
`-er`/`-est`, with spelling adjustments (`big → bigger`, `happy → happier`,
`nice → nicer`) — plus pronouns, possessives, and compact suffix-class lists that
generalize over endings and compounds (`mouse → mice`, so also `dormouse →
dormice`).

## When to use it

* **Has no data dependency** — logic only, no generated tables, no third-party
  crates. This keeps its footprint tiny.
* **Can be used standalone** for a smaller binary when a lightweight fallback is
  good enough.
* **Is not a correctness guarantee for arbitrary out-of-vocabulary words.** Some
  rules deliberately overgeneralize because this crate also serves as the
  extractor's prediction engine. The table-backed
  [`english`](https://crates.io/crates/english) crate consults Wiktionary-derived
  lookup tables first and should be preferred when attested irregulars matter.
* Inside this workspace it serves double duty: it is the runtime fallback for
  `english`, and the extractor uses it to classify attested forms as regular
  (droppable) or irregular (kept in the tables). Changing rules requires
  regenerating `english`'s generated data and rerunning accuracy checks.

All functionality lives in associated functions on the `EnglishCore` namespace
struct — there is no state to construct. Inputs are assumed to be lowercase
lemmas; case handling is the `english` crate's responsibility.

## Usage

```rust
use english_core::*;

// Nouns: regular pluralization with suffix-class rules.
assert_eq!(EnglishCore::pluralize_noun("cat"), "cats");
assert_eq!(EnglishCore::pluralize_noun("bus"), "buses");
assert_eq!(EnglishCore::pluralize_noun("mouse"), "mice");
assert_eq!(EnglishCore::noun("dog", &Number::Plural), "dogs");

// Adjective degree: short words take -er/-est, longer ones go periphrastic.
assert_eq!(EnglishCore::comparative("big"), "bigger");
assert_eq!(EnglishCore::superlative("happy"), "happiest");
assert_eq!(EnglishCore::comparative("beautiful"), "more beautiful");

// Verbs, pronouns, and possessives are covered too.
assert_eq!(
    EnglishCore::verb(
        "walk",
        &Person::Third,
        &Number::Singular,
        &Tense::Present,
        &Form::Finite,
    ),
    "walks",
);
assert_eq!(
    EnglishCore::pronoun(&Person::First, &Number::Singular, &Gender::Neuter, &Case::Possessive),
    "mine",
);
assert_eq!(
    EnglishCore::pronoun(
        &Person::First,
        &Number::Singular,
        &Gender::Neuter,
        &Case::PersonalPossessive,
    ),
    "my",
);
assert_eq!(EnglishCore::add_possessive("dog"), "dog's");
```

Because these are compact fallback rules, they are intentionally wrong for many
irregulars the table-backed crate would override (`go`, `child`, `good`, ...),
and they may overgeneralize for unknown words. Use `english` when you want the
attested table layer.

## Known limitations

* Case is not handled here; pass lowercase lemmas or normalize externally.
* Possessives use a simple trailing-`s` rule: `dogs'`, but also `bus'`/`James'`.
  Apply your own style guide if you prefer `bus's` or `James's`.
* Future tense and phrase-level grammar are periphrastic/compositional and are
  intentionally outside this crate.

## Installation

```bash
cargo add english-core
```

## License

- Code: Dual licensed under MIT and Apache © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)
  - [MIT](https://opensource.org/licenses/MIT)
  - [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
