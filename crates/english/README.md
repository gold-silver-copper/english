# english

[![Crates.io](https://img.shields.io/crates/v/english)](https://crates.io/crates/english)
[![Docs.rs](https://docs.rs/english/badge.svg)](https://docs.rs/english)
![License](https://img.shields.io/crates/l/english)
[![Discord](https://img.shields.io/discord/123456789012345678.svg?logo=discord&logoColor=white&color=5865F2)](https://discord.gg/tDBPkdgApN)

**english** is a fast, lightweight English inflection library written in Rust.
Total bundled data is about 1 MB. It provides highly accurate verb conjugation
and noun/adjective/adverb declension from processed Wiktionary data, making it
useful for real-time procedural text generation.

## ⚡ Speed and Accuracy

Evaluation of the English inflector (`cargo xtask accuracy`, using the
2026-07-02 Wiktionary dump) and performance benchmarking
(`examples/speedmark.rs`; measured rows averaged over 10 release runs) shows:

| Part of Speech | Correct / Total | Accuracy | Variant Gap | Throughput (calls/sec) | Time per Call |
|----------------|-----------------|----------|-------------|------------------------|---------------|
| **Nouns**      | 225900 / 225900 | 100.00% | 0           | 6,072,834              | 164.67 ns     |
| **Verbs**      | 151544 / 151544 | 100.00% | 1           | 9,549,311              | 104.73 ns     |
| **Adjectives** | 121548 / 121550 | 99.998% | 8           | 6,859,637              | 145.79 ns     |
| **Adverbs**    | 25123 / 25125   | 99.99%  | 2           | 11,144,625             | 89.73 ns      |

The accuracy percentages measure **recall through any published key**: the share
of plain attested Wiktionary slots reproducible via the bare lemma **or** any
`_n` sense key. They do not measure precision, nor whether the natural bare-lemma
call returns the primary/most-standard attested form.

For that natural-call view, `cargo xtask accuracy` also reports bare-lemma
correctness:

| Part of Speech | Bare Primary / Total | Bare Accuracy | Standard Form Demoted to `_n` | Over-generated |
|----------------|----------------------|---------------|-------------------------------|----------------|
| **Nouns**      | 223505 / 225900      | 98.94%        | 1655                          | 740            |
| **Verbs**      | 150787 / 151544      | 99.50%        | 626                           | 131            |
| **Adjectives** | 120924 / 121550      | 99.48%        | 624                           | 0              |
| **Adverbs**    | 24882 / 25125        | 99.03%        | 241                           | 0              |

Benchmarks are machine- and workload-dependent; run `cargo run -p english --example speedmark --release` on your target platform for local numbers.

### Breaking change in 0.4: spelling cleanup

The possessive-determiner enum variant is now `Case::PersonalPossessive`, and the
last-occurrence string helper is now `EnglishCore::replace_last_occurrence`.

### Breaking change in 0.3: underscore sense-key format

Sense-numbered keys now use the canonical underscore format (`die_2`, `lie_2`).
The pre-0.3 adjacent-digit spelling (`die2`, `lie2`) was removed because it is
ambiguous with ordinary digit-bearing words such as `mp3`, `F16`, and `F2`, which
are now always treated opaquely unless they are exact table keys.

## 📦 Installation

```bash
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
    ); // stole
    let object = count_with_number("potato", 7); // 7 potatoes

    let sentence = format!("The {} {} {}.", subject, verb, object);
    assert_eq!(sentence, "The running children stole 7 potatoes.");

    // --- Nouns ---
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    assert_eq!(English::noun("child", &Number::Plural), "children");
    // Sense-numbered keys expose homographs and attested variants.
    assert_eq!(English::noun("die_2", &Number::Plural), "dice");
    assert_eq!(count("man", 2), "men");
    assert_eq!(count_with_number("nickel", 3), "3 nickels");
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
    // Sense-numbered keys distinguish homographs: "lie" (recline) and "lie_2"
    // (tell an untruth) inflect differently.
    assert_eq!(
        English::verb(
            "lie",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lay"
    );
    assert_eq!(
        English::verb(
            "lie_2",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lied"
    );
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
    assert_eq!(English::adj("bad", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad", &Degree::Superlative), "worst");
    assert_eq!(English::adj("bad_2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad_3", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("bad_3", &Degree::Positive), "bad");

    // --- Adverbs ---
    assert_eq!(English::adverb("quickly", &Degree::Comparative), "more quickly");
    assert_eq!(English::adverb("well", &Degree::Comparative), "better");
    assert_eq!(English::adverb("badly", &Degree::Superlative), "worst");
    assert_eq!(English::adverb("fast", &Degree::Comparative), "faster");
    assert_eq!(English::adverb("early", &Degree::Superlative), "earliest");
    assert_eq!(English::adverb("far", &Degree::Comparative), "farther");
    assert_eq!(English::adverb("far_2", &Degree::Comparative), "further");

    // --- Pronouns ---
    assert_eq!(
        English::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Neuter,
            &Case::PersonalPossessive
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

For a more involved but still minimal example of building a small domain layer on
top of `english`, see `examples/semantic_triples.rs`:

```bash
cargo run -p english --example semantic_triples
```

It shows custom noun/verb/adj/adv types, semantic triples,
perspective-sensitive rendering, modifiers, complements, adjuncts, and
agreement-driven pronoun and tense shifts.

## Case handling

`english` accepts lowercase lemmas, but the public API also has a simple casing
convenience for common sentence text:

```rust
use english::{English, Number};

assert_eq!(English::noun("child", &Number::Plural), "children");
assert_eq!(English::noun("Child", &Number::Plural), "Children");
assert_eq!(English::noun("CHILD", &Number::Plural), "CHILDREN");
assert_eq!(English::noun("McDonald", &Number::Plural), "McDonalds");
```

Title-case and ALL-CAPS words may use lowercase table rows and then restore the
input style. Mixed case is deliberately not guessed, so proper-name-like tokens
such as `McDonald` fall through to the regular rule on the original spelling.
This is **not** semantic proper-noun or acronym detection: names, brands,
initialisms, and house-style casing may need caller-side normalization.

## Helper limitations

* `count` and `count_with_number` are small conveniences for `u32` counts. Exactly
  `1` is singular; every other value is plural. Decimal, negative, formatted, or
  localized quantities are caller responsibilities.
* `English::add_possessive` uses a simple trailing-`s` rule: `dogs'`, but also
  `bus'` and `James'`. Apply your own style guide if you prefer `bus's` or
  `James's`.

## 🔧 Crate Overview

### `english`

> The public API for verb conjugation and noun/adjective/adverb declension.

* Combines optimized data generated from `extractor` with inflection logic from
  `english-core`.
* Pure Rust; one third-party dependency (`phf`) plus the first-party
  `english-core`.
* PHF-backed irregular lookups with regular-rule fallback.
* Code generation ensures no runtime penalty.

### `english-core`

> The compact fallback / prediction engine for English inflection.

* Implements small rule approximations for conjugation/declension.
* Used by the extractor to classify forms as regular or irregular.
* Has no data dependency — logic-only.
* Can be used standalone for a smaller footprint, but is not guaranteed correct
  for arbitrary out-of-vocabulary words; use `english` for the table-backed API.

### `extractor`

> A tool to process and refine Wiktionary data.

* Parses large English Wiktionary dumps.
* Extracts verb, noun, adjective, and adverb forms.
* Uses `english-core` to filter out regular forms, preserving only irregulars.
* Numbers homograph senses **deterministically** by a pure sort of their emitted
  forms (no lockfile, no identity, no human review — see below).
* Emits every plain attested variant as its own sense-numbered key (`cactus_2` →
  *cacti*, `cactus_3` → …), numbered in form-signature order.
* Generates the static PHF tables used in `english`.
* `cargo xtask accuracy` measures both any-key reachability and bare-lemma
  primary correctness; run it before and after any rule or table change.

---

## 📦 Obtaining Wiktionary Data & Running the Extractor

This project relies on raw data extracted from Wiktionary. Current version built
with data from 2026-07-02.

- [Wiktextract (GitHub)](https://github.com/tatuylonen/wiktextract)
- [Kaikki.org raw data](https://kaikki.org/dictionary/rawdata.html)

### Steps

1. Download the **raw Wiktextract JSONL dump** (~20 GB) from
   [Kaikki.org](https://kaikki.org/dictionary/rawdata.html).
2. Place the file somewhere accessible (e.g. `../rawwiki.jsonl`).
3. From the repository root, run: `cargo xtask refresh-data --dump ../rawwiki.jsonl`.
4. The generated Rust tables are written to `crates/english/generated`; intermediate
   CSV/JSONL artifacts to `data/intermediate`.
5. Review `git diff crates/english/generated/`, then run
   `cargo xtask check-registry` before committing.

After committing regenerated tables, run `cargo xtask accuracy` to score them
against the dump. It measures the currently compiled committed tables, and needs
either the cached `data/intermediate/english_filtered.jsonl` or an explicit
`--dump /path/to/raw-wiktextract.jsonl`.

## Adverb degree

Adverbs use the same two-tier design as the other parts of speech (table first,
rule fallback) but a **different rule** from adjectives:

* **Adjectives** have a linguistically informed rule — short words take suffixal
  `-er`/`-est` (`fast → faster`), longer words go periphrastic (`beautiful → more
  beautiful`).
* **Adverbs** have a deliberately **conservative, unconditional periphrastic
  rule**: `EnglishCore::comparative_adverb("quickly") == "more quickly"`. On the
  dump, 99.1% of gradable adverbs take `more`/`most`, and a suffixal guess would
  be wrong far more often than right (`quicklier`, `abruptlier`, ...).

The small closed set of adverbs that inflect otherwise is table-driven, exactly
like irregular nouns/verbs:

* flat adverbs (homographs of their adjective): `fast → faster`, `hard → harder`,
  `early → earlier`, `late → later`;
* suppletives: `well → better`, `badly → worse`, `far → farther` (with
  `far_2 → further`);
* locational/directional adverbs inflect with `farther/further`: `downhill →
  farther downhill`, `east → farther east`.

Where an adverb attests both the periphrastic and a single-word form, the
periphrastic wins the bare key (so `quickly → more quickly`) and the single-word
form is a numbered key (`quickly_2 → quicklier`). This is correct for the large
`-ly` class; the handful of flat adverbs that also list `more X` (e.g. `deep`) get
the periphrastic on the bare key and the suffixal at `deep_2 → deeper` — both stay
reachable, so any-key accuracy is unaffected.

**Accuracy:** 25123 / 25125 slots (99.99%); the two residuals are malformed
Wiktionary forms (`more ... humouredly` / `most ... humouredly`). See
`English::adverb` and `EnglishCore::adverb` for the API.

## Deterministic sense numbering

Homographs that inflect differently share a lemma and are disambiguated by a
numeric suffix (`lie` → *lay*, `lie_2` → *lied*; `die_2` → *dice*). The suffix is
assigned by a **pure, transparent sort** of the forms — no lockfile, no frozen
identity, no human-review workflow.

For each `(lemma, part of speech)` the extractor:

1. gathers every plain attested inflection pattern and drops the one the regular
   rule engine already produces (so the rule serves it at runtime);
2. sorts the survivors by emitted **form signature** (standard senses before
   slang/soft ones, then lexicographically);
3. hands out suffixes: if a regular form was dropped, the bare key is reserved
   for the rule engine and numbering starts at `_2`; otherwise the first-sorted
   survivor takes the bare lemma. The rest number upward (`_2`, `_3`, …).

Because the key is a function of the forms alone, **reordering the dump's entries
can never change the output** — generation is reproducible. `cargo xtask
check-registry` is a dump-free consistency gate: it verifies the committed tables
are well-formed, have unique and correctly shaped keys, no empty columns, and
preserve rule/table layering. It deliberately does **not** verify that a row's
irregular values are correct — those are attested data, not derivable without the
dump. `cargo xtask accuracy` (with the dump) is the authoritative value check.

**Stability guarantee — "fairly stable", not frozen.** Keys are deterministic but
**not immutable**. If Wiktionary adds, removes, or edits a lemma's attested forms,
the sort can renumber that lemma's `_<n>` keys — a lexicographically earlier new
variant deliberately shifts later ones up. Do not persist `_n` keys as stable
semantic IDs across data refreshes. What stays true is that the set of forms a
lemma exposes is reachable through some key, and that a slang-only sense never
takes the bare key from a standard one. Review the `crates/english/generated/`
diff on a refresh as you would any regenerated artifact.

Current runtime lookup is permissive for compatibility: a `_<digits>` suffix is
stripped when it resolves to a real table key or to a tabled base lemma. That
means a nonsensical key like `child_999` may behave like `child` rather than being
reported invalid. If your application needs strict key validation, keep your own
allow-list from the versioned generated tables until a strict public API exists.

## Benchmarks

Performance benchmarks were run on an M2 MacBook.

Benchmarking this kind of project requires opinionated decisions: many words have
alternative inflections, Wiktionary data is imperfect, and countability tags can
be inconsistent. Treat bundled numbers as a baseline, take them with a grain of
salt, and benchmark your own use cases. Suggestions to improve benchmarking are
welcome.

## Disclaimer

Wiktionary data is unstable and subject to upstream changes. The generated lookup
tables in `crates/english/generated/*_phf.rs` are the source of truth for a given
revision. Sense-numbered keys (`lie_2`, `die_2`, …) are deterministic for a dump
but may be renumbered when upstream forms change — see
[Deterministic sense numbering](#deterministic-sense-numbering).

## Inspirations and Thanks

- Ole in the Bevy Discord suggested `phf` instead of sorted arrays, which resulted
  in up to 40% speedups.
- <https://github.com/atteo/evo-inflector>
- <https://github.com/plurals/pluralize>

## 📄 License

- Code: Dual licensed under MIT and Apache © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)
  - [MIT](https://opensource.org/licenses/MIT)
  - [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

- Data: Wiktionary content is dual-licensed under
  - [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
  - [GNU FDL](https://www.gnu.org/licenses/fdl-1.3.html)
