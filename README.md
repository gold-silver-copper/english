# english

[![Crates.io](https://img.shields.io/crates/v/english)](https://crates.io/crates/english)
[![Docs.rs](https://docs.rs/english/badge.svg)](https://docs.rs/english)
![License](https://img.shields.io/crates/l/english)
[![Discord](https://img.shields.io/discord/123456789012345678.svg?logo=discord&logoColor=white&color=5865F2)](https://discord.gg/tDBPkdgApN)


**english** is a blazing fast and light weight English inflection library written in Rust. Total bundled data size is less than 1 MB. It provides extremely accurate verb conjugation and noun/adjective declension based on highly processed Wiktionary data, making it ideal for real-time procedural text generation.

## ⚡ Speed and Accuracy

Evaluation of the English inflector (`cargo xtask accuracy`, using the 2026-07-02 Wiktionary dump) and performance benchmarking (`examples/speedmark.rs`) shows:

| Part of Speech | Correct / Total | Accuracy  | Variant Gap | Throughput (calls/sec) | Time per Call |
|----------------|----------------|-----------|-------------|-----------------------|---------------|
| **Nouns**      | 225900 / 225900 | 100.00%  | 0           | 7,499,749             | 133 ns        |
| **Verbs**      | 151540 / 151540 | 100.00%  | 0           | 12,423,891            | 80 ns         |
| **Adjectives** | 121548 / 121550 | 100.00%  | 8           | 15,607,807            | 64 ns         |

*Note: Benchmarking was done under a worst-case scenario; typical real-world usage is 50~ nanoseconds faster.*

## 📦 Installation

```
cargo add english
```

Then in your code:

```rust
use english::*;
fn main() {
    // --- Mixed Sentence Example ---
    let subject_number = Number::Plural;
    let subject = format!(
        "{} {}",
        English::verb(
            "run",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle
        ),
        English::noun("child", &subject_number)
    ); // running children
    let verb = English::verb(
        "steal",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    ); //stole
    let object = count_with_number("potato", 7); //7 potatoes

    let sentence = format!("The {} {} {}.", subject, verb, object);
    assert_eq!(sentence, "The running children stole 7 potatoes.");

    // --- Nouns ---
    assert_eq!(
        format!("{} of jeans", count_with_number("pair", 3)),
        "3 pairs of jeans"
    );
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    // Sense-numbered keys expose homographs and attested variants.
    assert_eq!(English::noun("die_2", &Number::Plural), "dice");
    // Use count function for better ergonomics if needed
    assert_eq!(count("man", 2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(count_with_number("nickel", 3), "3 nickels");
    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // --- Verbs ---
    assert_eq!(
        English::verb(
            "pick",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "picked"
    );
    assert_eq!(
        English::verb(
            "walk",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle
        ),
        "walking"
    );
    assert_eq!(
        English::verb(
            "go",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Participle
        ),
        "gone"
    );
    // Sense-numbered keys distinguish homographs: "lie" (tell an untruth)
    // and "lie_2" (recline) inflect differently.
    assert_eq!(
        English::verb(
            "lie",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lied"
    );
    assert_eq!(
        English::verb(
            "lie_2",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lay"
    );
    // "to be" has the most verb forms in english and requires using verb()
    assert_eq!(
        English::verb(
            "be",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite
        ),
        "am"
    );

    // --- Adjectives ---
    // Numbered adjective keys expose standard, slang, and periphrastic variants.
    assert_eq!(English::adj("bad", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad", &Degree::Superlative), "worst");
    assert_eq!(English::adj("bad_2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad_2", &Degree::Superlative), "baddest");
    assert_eq!(English::adj("bad_3", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("bad_3", &Degree::Superlative), "most bad");
    assert_eq!(English::adj("bad_3", &Degree::Positive), "bad");

    // --- Pronouns ---
    assert_eq!(
        English::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Neuter,
            &Case::PersonalPossesive
        ),
        "my"
    );
    assert_eq!(
        English::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Neuter,
            &Case::Possessive
        ),
        "mine"
    );

    // --- Possessives ---
    assert_eq!(English::add_possessive("dog"), "dog's");
    assert_eq!(English::add_possessive("dogs"), "dogs'");
}
```

---

For a more involved but still minimal example of building a small domain layer on top of `english`, see `crates/english/examples/semantic_triples.rs`:

```bash
cargo run -p english --example semantic_triples
```

It shows custom noun/verb/adj/adv types, semantic triples, perspective-sensitive rendering, modifiers, complements, adjuncts, and agreement-driven pronoun and tense shifts.

## 🔧 Crate Overview

### `english`

> The public API for verb conjugation and noun/adjective declension.

* Combines optimized data generated from `extractor` with inflection logic from `english-core`
* Pure Rust, only one dependency: phf
* PHF-backed irregular lookups with regular-rule fallback
* Code generation ensures no runtime penalty

### `english-core`

> The core engine for English inflection — pure algorithmic logic.

* Implements the core rules for conjugation/declension
* Used to classify forms as regular or irregular for the extractor
* Has no data dependency — logic-only
* Can be used stand alone for an even smaller footprint (at the cost of some accuracy)

### `extractor`

> A tool to process and refine Wiktionary data.

* Parses large English Wiktionary dumps
* Extracts all verb, noun, and adjective forms
* Uses `english-core` to filter out regular forms, preserving only irregulars
* Resolves homograph sense numbers against the checked-in assignment lockfiles so keys stay stable across releases
* Generates the static PHF tables used in `english`
* Emits every plain attested variant as its own sense-numbered key (`cactus_2` → *cacti*, `cactus_3` → …), numbered canonical-first by the head template's listing order; each variant key is pinned in the lockfiles like any other
* `cargo xtask accuracy` measures, per part of speech, the share of slots whose plain attested Wiktionary forms the library reproduces, plus the variant gap (attested forms not reachable through any published key) — run it before and after any rule or table change. It needs dump data: either the cached `data/intermediate/english_filtered.jsonl` (gitignored; produced by a prior `cargo xtask refresh-data`) or an explicit `--dump /path/to/raw-wiktextract.jsonl`

---

## 📦 Obtaining Wiktionary Data & Running the Extractor

This project relies on raw data extracted from Wiktionary. Current version built with data from 2026-07-02.

- [Wiktextract (GitHub)](https://github.com/tatuylonen/wiktextract)
- [Kaikki.org raw data](https://kaikki.org/dictionary/rawdata.html)

### Steps

1. Download the **raw Wiktextract JSONL dump** (~20 GB) from [Kaikki.org](https://kaikki.org/dictionary/rawdata.html).
2. Place the file somewhere accessible (e.g. `../rawwiki.jsonl`).
3. From the repository root, run: `cargo xtask refresh-data --dump ../rawwiki.jsonl --data-date YYYY-MM-DD` (use the dump's date)
4. The generated Rust tables are written to `/crates/english/generated`, the assignment lockfiles to `/data/assignments`, and intermediate CSV/JSONL artifacts to `/data/intermediate`.
5. Review `git diff data/assignments/` like a `Cargo.lock` diff, then run `cargo xtask check-registry` before committing.

To also run the extractor evaluation reports against the current library data, add `--with-checks`.

## 🔒 Deterministic sense numbering

Homographs that inflect differently share a lemma and are disambiguated by a numeric suffix
(`lie` → *lied*, `lie_2` → *lay*; `die_2` → *dice*). Historically that suffix was **positional** —
assigned by alphabetically sorting whatever forms survived filtering in a given dump — so any
upstream change could renumber a lemma and silently swap the meaning of a published key.

The suffix is now pinned in checked-in **assignment lockfiles** (`data/assignments/{noun,verb,adj}.lock.csv`),
the data equivalent of `Cargo.lock`. The matching model is **exact match or human decision** —
resolution never guesses:

* Each emitted key is anchored to a stable identity and a **frozen** suffix. Two anchor tiers
  exist: an author-authored Wiktionary **senseid** (`sid:` — durable; the one identity that
  carries a key through form edits) or, for everything else, the **form signature** (`sig:` —
  the identity *is* the forms, so a matched key can never change what it emits).
* On every refresh, senses are matched to existing lock rows **only on exact anchor equality**;
  genuinely new senses get **append-only** higher suffixes; vanished senses are **tombstoned**
  and their suffix is retired forever. When a retired row and a new sense look like a possible
  rematch (e.g. an upstream spelling correction), the refresh writes a **rematch proposal** to
  `data/intermediate/pending-rematches.csv` instead of guessing; a human sanctions it by adding
  the corresponding row to `data/assignments/overrides.csv`, and the next refresh applies it —
  the tool proposes, the reviewer decides, git records why.
* `cargo xtask check-registry` fails the build if any previously-committed key changed meaning:
  anchors, suffixes, and **forms** are immutable (bare `sid:` drift and sanctioned overrides are
  the only exceptions), and the committed PHF tables must correspond 1:1 to the active lock rows.
* `cargo xtask report-coverage` shows how much of each lockfile rests on each anchor tier.

These initial lockfiles were generated during the pre-merge determinism work after fixing the extractor/table policy bugs. That means the first PR is allowed to re-key the previously generated tables: `origin/main` had no assignment lockfiles yet, so the checked-in locks in this change become the first stability baseline. After they land, ordinary refreshes must preserve key meanings within the same `DATA_MAJOR`.

`cargo xtask seed-assignments` remains available as a one-time bootstrap command for freezing an existing set of committed PHF tables into lockfiles, but it is not part of the normal refresh flow.

### Stability policy: immutable within a data-major, compacted across them

Left alone forever, an append-only registry accumulates cruft: tombstoned rows, suffix gaps
from ambiguous drift, ever-higher numbers for new senses. The policy that bounds this is
versioned like semver:

* **Within a data-major version** (tracked in `data/assignments/DATA_MAJOR`): published keys
  never change meaning. Refreshes may update forms in place, append new suffixes, and
  tombstone vanished senses — `cargo xtask check-registry` hard-fails anything else.
* **At a data-major release**: run `cargo xtask compact`. It deletes all tombstones, closes
  suffix gaps (preserving relative order; the bare slot is never newly occupied), regenerates
  the tables, and bumps `DATA_MAJOR`. `check-registry` recognizes the bump and waives
  cross-version immutability for exactly that PR — the bump in the diff *is* the reviewable
  declaration of a breaking data change. Ship it as a **semver-major** crate release and list
  the renumbered keys in the changelog.

So sense keys are stable for as long as you stay on a crate major version, and cruft is
bounded by the compaction cadence instead of growing forever.

What the guarantee covers: the mapping from a published key to its sense (and therefore
which inflection family it produces). What it does not cover: the exact form values,
which update in place as Wiktionary is edited.

## Benchmarks
Performance benchmarks were run on my M2 Macbook.

Writing benchmarks and tests for such a project is rather difficult and requires opinionated decisions. Many words may have alternative inflections, and the data in wiktionary is not perfect. Many words might be both countable and uncountable, the tagging of words may be inconsistent. This library includes a few uncountable words in its dataset, but not all. Uncountable words require special handling anyway. Take all benchmarks with a pound of salt, write your own tests for your own usecases. Any suggestions to improve the benchmarking are highly appreciated.

## Disclaimer
Wiktionary data is often unstable and subject to weird changes. This means that the provided inflections may change unexpectedly. The generated lookup tables in `crates/english/generated/*_phf.rs` are the source of truth for a given revision, and the assignment lockfiles in `data/assignments/*.lock.csv` keep each sense-numbered key (`lie_2`, `die_2`, …) pointing at the same inflection across revisions — see [Deterministic sense numbering](#-deterministic-sense-numbering).

## Inspirations and Thanks
- Ole in the bevy discord suggested I use ```phf``` instead of sorted arrays, this resulted in up to 40% speedups
- https://github.com/atteo/evo-inflector
- https://github.com/plurals/pluralize


## 📄 License

- Code: Dual licensed under MIT and Apache © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)
  - [MIT](https://opensource.org/licenses/MIT)
  - [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

- Data: Wiktionary content is dual-licensed under
  - [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
  - [GNU FDL](https://www.gnu.org/licenses/fdl-1.3.html)
