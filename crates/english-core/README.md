# english-core

The dependency-free **regular** rule engine for English inflection — the pure
algorithmic core underneath the [`english`](https://crates.io/crates/english)
crate.

`english-core` knows nothing about Wiktionary or lookup tables. It encodes the
productive rules of English morphology — add `-s`, `-ed`, `-er`/`-est`, with
their spelling adjustments (`big → bigger`, `happy → happier`, `nice → nicer`) —
plus pronouns, possessives, and a few compact suffix-class lists that generalize
over endings and compounds (`mouse → mice`, so also `dormouse → dormice`).

## When to use it

* **Has no data dependency** — logic only, no generated tables, no third-party
  crates. This keeps its footprint tiny.
* **Can be used standalone** for an even smaller binary, at the cost of some
  accuracy: without the irregular tables it will regularize words English does
  not (`go → goed`, `child → childs`). If you want attested-correct irregulars,
  use the table-backed [`english`](https://crates.io/crates/english) crate, which
  consults its lookup tables first and falls back to this engine.
* Inside this workspace it also serves double duty: it is the runtime fallback
  for `english`, and the extractor uses it to classify attested forms as regular
  (droppable) or irregular (kept in the tables).

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
assert_eq!(EnglishCore::add_possessive("dog"), "dog's");
```

Because these are the *regular* rules, they are correct for productive words but
deliberately wrong for irregulars the tables would override.

## Installation

```
cargo add english-core
```

## License

- Code: Dual licensed under MIT and Apache © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)
  - [MIT](https://opensource.org/licenses/MIT)
  - [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
