# `english-phrase` implementation plan

## Purpose

This plan is focused directly on implementing the X-bar / DP-hypothesis system in `english-phrase`.

It is not a generic architecture note. Each phase below maps to one or more of these target rules:

- `XP -> (Specifier) X'`
- `X' -> X (Complement)`
- `X' -> X' Adjunct`
- `X' -> Adjunct X'`
- `CP -> (Specifier) C'`
- `C' -> C TP`
- `TP -> DP T'`
- `T' -> T VP`
- `T' -> T vP`
- `vP -> DP v'`
- `v' -> v VP`
- `VP -> V'`
- `V' -> V (DP/PP/AP/CP)`
- `DP -> (DP) D'`
- `D' -> D NP`
- `NP -> N'`
- `N' -> N (PP/CP)`
- `PP -> P'`
- `P' -> P DP`
- `AP -> (AdvP) A'`
- `A' -> A (PP/CP)`
- `AdvP -> (AdvP) Adv'`
- `Adv' -> Adv (PP/CP)`

The implementation priority is to make those rules real in code, while keeping the API strongly typed and pleasant to use.

## Progress snapshot

Current implementation status in [lib.rs](/Users/kisaczka/Desktop/programming/english/crates/english-phrase/src/lib.rs):

- completed:
  - public `PrepositionalPhrase`
  - public `AdverbPhrase`
  - structured PP and AdvP rendering
  - PP integration into adjective, nominal, and clause-level material
- now implemented:
  - `NominalPhrase` as the noun-headed nominal core
  - `DeterminerPhrase` as the outer nominal layer
  - clause and agreement paths updated to operate on `DeterminerPhrase`
  - real `Pronoun` support for DP rendering and agreement
  - proper-name DPs
  - possessor rendering in `Spec,DP`
  - typed nominal postmodifiers via `NominalPostmodifier`
  - explicit `RelativeClause` support for future CP-based nominal postmodification
- still missing:
  - explicit CP
  - explicit TP
  - explicit vP
  - explicit complement vs adjunct split inside VP
  - fully typed CP integration beyond relative-clause scaffolding

## Current status in `lib.rs`

As of now, [lib.rs](/Users/kisaczka/Desktop/programming/english/crates/english-phrase/src/lib.rs) already has:

- a public `PrepositionalPhrase`
- a public `AdverbPhrase`
- a public `AdjPhrase`
- a public `NominalPhrase`
- a public `DeterminerPhrase`
- a public `VerbPhrase`
- a public `Clause`
- a public `Sentence`

What is still missing for genuine X-bar support:

- no explicit `CP`
- no explicit `TP`
- no explicit `vP`
- no explicit head/specifier/complement/adjunct structure in the types
- `Clause` is still a convenience sentence-level shell, not a real clausal projection stack
- AP, AdvP, NP, and VP still mix complements and adjunct-like material more loosely than the theory wants

So the next work should not be “add more phrase helpers.” It should be “make the existing phrase objects line up with the rewrite rules.”

## Type-safety principles

These are the constraints that should guide every phase.

### 1. Phrase category distinctions must be real Rust types

Use separate public types for:

- `DeterminerPhrase`
- `NominalPhrase`
- `PrepositionalPhrase`
- `AdjectivePhrase`
- `AdverbPhrase`
- `VerbPhrase`
- `TensePhrase`
- `ComplementizerPhrase`
- `NonFiniteClause`

Do not encode those as one generic phrase type with tags.

### 2. Complement vs adjunct must be different in the type system

For every major projection, use separate enums or fields for:

- complements
- adjuncts

Do not use a single `Vec<Something>` for both.

### 3. DP vs NP must be different types

This is the core DP-hypothesis requirement.

- `DP` is the maximal nominal projection
- `NP` is the nominal core

The nominal layer has to stay split into separate public types if we want real support for:

- `DP -> (DP) D'`
- `D' -> D NP`
- `NP -> N'`
- `N' -> N (PP/CP)`

### 4. Finite vs non-finite clause must be different types

If we want real support for:

- `CP -> (Specifier) C'`
- `C' -> C TP`
- `TP -> DP T'`
- `T' -> T VP`

then finite clauses and non-finite clauses should not be one undifferentiated `Clause`.

### 5. Escape hatches should exist, but stay visibly unstructured

Keep raw-string escape hatches explicit:

```rust
pub enum NominalPostmodifier {
    PrepositionalPhrase(PrepositionalPhrase),
    RelativeClause(RelativeClause),
    Raw(String),
}
```

That makes it obvious what is genuine syntax and what is fallback text.

## Phase 1: make PP and AdvP genuinely X-bar-like

### Rules targeted

- `PP -> P'`
- `P' -> P DP`
- `AdvP -> (AdvP) Adv'`
- `Adv' -> Adv (PP/CP)`

### Why this phase comes first

`PrepositionalPhrase` and `AdverbPhrase` now exist, but they are still transitional:

- `PrepositionalPhrase` can target `DeterminerPhrase`, but not yet `ComplementizerPhrase`
- `AdverbPhrase` has `modifiers` and `complements`, but not a specifier/head/complement framing
- neither type is yet embedded in a broader projection system

### Target end state

```rust
pub struct PrepositionalPhrase {
    head: Preposition,
    complement: PrepositionalComplement,
}

pub enum PrepositionalComplement {
    DeterminerPhrase(Box<DeterminerPhrase>),
    ComplementizerPhrase(Box<ComplementizerPhrase>),
    NonFiniteClause(Box<NonFiniteClause>),
    Raw(String),
}

pub struct AdverbPhrase {
    specifier: Option<Box<AdverbPhrase>>,
    head: Adverb,
    complements: Vec<AdverbComplement>,
    adjuncts: Vec<AdverbAdjunct>,
}
```

### Type-safety requirements

- prepositions should stop being bare `String` heads eventually
- PP complement category should be typed
- `AdvP` specifier-like material must be distinguishable from complements

### Checklist

- add `Preposition`
- add `Adverb`
- rename current `Text(...)` enum variants to `Raw(...)`
- convert `PrepositionalPhrase` from `String + complement` into a head/complement structure
- split `AdverbPhrase` into:
  - optional specifier
  - head
  - complements
  - adjuncts if needed
- update tests so PP and AdvP are no longer just “string with children”

### Progress

- public `PrepositionalPhrase` implemented
- public `AdverbPhrase` implemented
- PP recursion implemented and tested
- AdvP complements implemented and tested
- still pending:
  - typed `Preposition`
  - typed `Adverb`
  - explicit specifier field on `AdverbPhrase`
  - renaming remaining `Text(...)` enum variants to `Raw(...)`

## Phase 2: split the nominal layer into `DeterminerPhrase` and `NominalPhrase`

### Rules targeted

- `DP -> (DP) D'`
- `D' -> D NP`
- `NP -> N'`
- `N' -> N (PP/CP)`
- `N' -> AP N'`
- `N' -> N' PP`
- `N' -> N' CP`

### Why this is the most important phase

This is the single structural change that makes the crate genuinely DP-based instead of merely DP-aware in docs.

The old nominal surface bundled:

- determiner
- quantity
- modifiers
- complements
- noun head

That obscures the actual rewrite system.

### Target end state

```rust
pub struct DeterminerPhrase {
    specifier: Option<Box<DeterminerPhrase>>, // possessor in Spec,DP
    head: DeterminerHead,
    complement: NominalPhrase,
}

pub struct NominalPhrase {
    adjuncts: Vec<NominalAdjunct>,
    head: Noun,
    complements: Vec<NominalComplement>,
    postmodifiers: Vec<NominalPostmodifier>,
}
```

Ergonomic surface shape is also acceptable:

```rust
pub struct DeterminerPhrase {
    determiner: Option<Determiner>,
    possessor: Option<Box<DeterminerPhrase>>,
    nominal: NominalPhrase,
    quantity: Quantity,
}
```

### Required DP-specific features

- possessor in `Spec,DP`
- pronoun DPs
- proper-name DPs
- null or implicit D behavior where needed

### Type-safety requirements

- `DeterminerPhrase` and `NominalPhrase` must be different public types
- pronouns should not be forced through noun-headed structure
- proper names should not be forced through noun-headed structure
- nominal complements and postmodifiers must be typed

### Checklist

- add `DeterminerPhrase`
- add `NominalPhrase`
- move determiners out of the old monolithic nominal type
- move possessive support into `DeterminerPhrase`
- move noun-headed complement logic into `NominalPhrase`
- make AP premodification attach to the nominal layer, not the DP layer
- make PP and CP postmodification attach to the nominal layer
- migrate existing nominal tests into DP/NP tests

### Progress

- `DeterminerPhrase` implemented
- `NominalPhrase` implemented
- determiner-bearing render path now lives at the `DeterminerPhrase` level
- noun-headed render path now lives at the `NominalPhrase` level
- nominal agreement now comes from `NominalPhrase`
- pronoun DP constructors implemented
- proper-name DP constructors implemented
- possessor / `Spec,DP` rendering implemented
- typed nominal postmodifiers implemented via `NominalPostmodifier`
- typed relative clauses implemented via `RelativeClause`
- still pending:
  - replacing string-backed relative clauses with full CP objects
  - separating nominal complements from nominal adjuncts more strictly
  - tightening the DP/NP API around head/specifier/complement terminology

## Phase 3: reshape AP around the X-bar rule

### Rules targeted

- `AP -> (AdvP) A'`
- `A' -> A (PP/CP)`
- `A' -> AdvP A'`
- `A' -> A' PP`

### Why this matters

Current `AdjPhrase` is close, but still too flat:

- `degree`
- `intensifier`
- `complements`

That is useful, but not yet a direct implementation of AP as a projection.

### Target end state

```rust
pub struct AdjectivePhrase {
    specifier: Option<Box<AdverbPhrase>>,
    head: Adj,
    complements: Vec<AdjectiveComplement>,
    adjuncts: Vec<AdjectiveAdjunct>,
}
```

### Type-safety requirements

- AP specifier/intensifier should be structurally separate from complements
- AP complements should be typed:
  - `PP`
  - `CP`
  - maybe `NonFiniteClause`
- keep comparative/superlative morphology on the adjective head, not by flattening phrase structure

### Checklist

- replace `degree + intensifier` framing with a projection-aware AP model
- allow `AdvP` as specifier
- keep adjective morphology via `Adj`
- split AP complements from AP adjuncts

## Phase 4: reshape VP around the X-bar rule

### Rules targeted

- `VP -> V'`
- `V' -> V (DP/PP/AP/CP)`
- `V' -> V' PP`
- `V' -> V' AdvP`

### Why this matters

Current `VerbPhrase` still mainly models:

- head verb
- tense
- aspect
- polarity
- modal
- particle
- agreement

That is good for realization, but not yet a true X-bar VP.

### Target end state

```rust
pub struct VerbPhrase {
    head: Verb,
    particle: Option<Particle>,
    complements: Vec<VerbComplement>,
    adjuncts: Vec<VerbAdjunct>,
    aspect: Aspect,
    polarity: Polarity,
}

pub enum VerbComplement {
    DeterminerPhrase(DeterminerPhrase),
    PrepositionalPhrase(PrepositionalPhrase),
    AdjectivePhrase(AdjectivePhrase),
    ComplementizerPhrase(ComplementizerPhrase),
    NonFiniteClause(NonFiniteClause),
}

pub enum VerbAdjunct {
    PrepositionalPhrase(PrepositionalPhrase),
    AdverbPhrase(AdverbPhrase),
    Raw(String),
}
```

### Type-safety requirements

- complements and adjuncts must not share one field
- particles should have their own type
- clause complements must be structurally distinct from PP and DP complements

### Checklist

- add `VerbComplement`
- add `VerbAdjunct`
- move object/PP/AP/CP material into complements
- move adverbials and VP-attached PPs into adjuncts
- preserve the existing auxiliary-chain realization pipeline, but make it operate over this richer VP

## Phase 5: introduce TP as the main finite clause projection

### Rules targeted

- `TP -> DP T'`
- `T' -> T VP`

### Why this matters

Current `Clause` is still a convenience wrapper:

- subject
- predicate
- object
- prepositionals

That is useful for output, but it is not a TP.

### Target end state

```rust
pub struct TensePhrase {
    specifier: DeterminerPhrase,
    head: TenseHead,
    complement: VerbPhrase,
}
```

More ergonomic public shape:

```rust
pub struct TensePhrase {
    subject: DeterminerPhrase,
    tense: Tense,
    predicate: VerbPhrase,
}
```

### Type-safety requirements

- finite subject position must be explicit
- TP should be the owner of finite agreement decisions
- `Sentence` should consume clause/projection types, not raw rendered text

### Checklist

- add `TensePhrase`
- move finite agreement and auxiliary choice to TP-driven rendering
- refactor current `Clause` to sit above or beside TP

## Phase 6: add internal vP support

### Rules targeted

- `TP -> DP T'`
- `T' -> T vP`
- `vP -> DP v'`
- `v' -> v VP`

### Why this matters

This is the main contemporary variant on top of the simpler TP/VP system.

It should probably be supported internally even if the public API remains more ergonomic.

### Target end state

```rust
pub struct LightVerbPhrase {
    specifier: DeterminerPhrase,
    head: LightVerb,
    complement: VerbPhrase,
}
```

### Type-safety requirements

- external argument introduction should be distinct from lexical VP structure
- keep `vP` mostly internal unless there is a compelling public use case

### Checklist

- add internal `LightVerbPhrase`
- make `TensePhrase` optionally target `vP` internally
- use `vP` where it helps with causatives, transitives, and later passive support

## Phase 7: add CP and complementizer structure

### Rules targeted

- `CP -> (Specifier) C'`
- `C' -> C TP`

### Why this matters

Without CP, the crate cannot model:

- complement clauses
- relative clauses
- many nominal and verbal postmodifiers/complements

### Target end state

```rust
pub struct ComplementizerPhrase {
    specifier: Option<ClauseSpecifier>,
    head: Complementizer,
    complement: TensePhrase,
}
```

### Type-safety requirements

- complementizer-headed clauses should not be encoded as raw strings
- relative clauses should be a typed structure, not text postmodifiers

### Checklist

- add `Complementizer`
- add `ComplementizerPhrase`
- add `RelativeClause`
- let NP postmodifiers include relative `CP`
- let VP/AP take typed clausal complements where appropriate

## Phase 8: add non-finite clauses

### Rules targeted

- support clausal complements that are not finite TP/CP
- make room for the same X-bar layering in infinitival and participial structures

### Why this matters

The rule inventory you listed is finite-clause-centered, but English phrase structure needs non-finite clauses for:

- `the decision to leave`
- `ready to leave`
- `before leaving`

### Target end state

```rust
pub struct NonFiniteClause {
    form: NonFiniteForm,
    predicate: VerbPhrase,
    subject: Option<DeterminerPhrase>,
}
```

### Type-safety requirements

- finite and non-finite clauses must be different types
- AP/PP/NP/VP complements should be able to require one or the other explicitly

### Checklist

- add `NonFiniteClause`
- add infinitival and participial constructors
- integrate into AP, PP, NP, and VP complement inventories

## Phase 9: refactor `Clause` and `Sentence` around projections

### Rules targeted

- turn the current sentence-level convenience API into a thin layer over `TP`, `CP`, and `NonFiniteClause`

### Why this matters

Once `TP` and `CP` exist, the current `Clause` type should stop being the primary syntax object.

### Target end state

```rust
pub enum Clause {
    TensePhrase(TensePhrase),
    ComplementizerPhrase(ComplementizerPhrase),
    NonFiniteClause(NonFiniteClause),
}

pub struct Sentence {
    clause: Clause,
    capitalize: bool,
    terminal: Terminal,
}
```

### Checklist

- make `Sentence` consume typed clause projections
- keep punctuation/capitalization above syntax
- keep `Clause` as a tagged wrapper if it still helps ergonomics

## Testing plan aligned to the rewrite rules

Each phase should add tests that correspond directly to the rule being implemented.

### DP / NP tests

- `DP -> D NP`
- `DP -> DP D'` for possessors
- `DP -> D` for pronouns
- `DP -> D NP[Name]` or equivalent for proper names
- `N' -> AP N'`
- `N' -> N' PP`
- `N' -> N' CP`

### VP tests

- `V' -> V DP`
- `V' -> V PP`
- `V' -> V AP`
- `V' -> V CP`
- `V' -> V' PP`
- `V' -> V' AdvP`

### AP / AdvP tests

- `AP -> (AdvP) A'`
- `A' -> A PP`
- `A' -> A CP`
- `AdvP -> (AdvP) Adv'`
- `Adv' -> Adv PP`

### Clause tests

- `TP -> DP T'`
- `T' -> T VP`
- `CP -> C TP`
- `vP -> DP v'`
- `v' -> v VP`

## Recommended implementation order

1. Reshape `PrepositionalPhrase` and `AdverbPhrase` into true X-bar phrase types
2. Split the nominal layer into `DeterminerPhrase` and `NominalPhrase`
3. Reshape `AdjPhrase` into a projection-aware `AdjectivePhrase`
4. Reshape `VerbPhrase` into a true VP with complements and adjuncts
5. Add `TensePhrase`
6. Add internal `LightVerbPhrase`
7. Add `ComplementizerPhrase`
8. Add `NonFiniteClause`
9. Refactor `Clause` and `Sentence` to sit above those projections

## End-state API sketch

```rust
let dp = DeterminerPhrase::from_noun("child")
    .determiner(Determiner::the())
    .postmodifier(
        RelativeClause::who(
            ComplementizerPhrase::new(
                Complementizer::null(),
                TensePhrase::new(
                    DeterminerPhrase::gap(),
                    VerbPhrase::new("wait").past(),
                ),
            ),
        ),
    )
    .plural();

let vp = VerbPhrase::new("steal")
    .complement(
        VerbComplement::DeterminerPhrase(
            DeterminerPhrase::from_noun("potato").count(7),
        ),
    )
    .adjunct(
        VerbAdjunct::PrepositionalPhrase(
            PrepositionalPhrase::new(
                "from",
                DeterminerPhrase::from_noun("market").the(),
            ),
        ),
    );

let tp = TensePhrase::new(dp, vp).past();
let sentence = Sentence::new(Clause::TensePhrase(tp)).capitalize().period();
```

That is the target:

- directly grounded in the rewrite rules
- strongly typed
- projection-aware
- still pleasant to use
