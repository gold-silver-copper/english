# `english-phrase` roadmap

## Purpose

This document turns the findings in [RESEARCH.md](/Users/kisaczka/Desktop/programming/english/crates/english-phrase/RESEARCH.md) into a concrete implementation plan for a **strictly X-bar, DP-first English surface realizer**.

This roadmap deliberately stays inside one framework:

- headed projections
- specifier / complement / adjunct distinctions
- DP over NP
- clause structure as projections above VP

It is not a mixed-theory roadmap. It is an X-bar roadmap.

## Core architectural commitments

- `english` remains the morphology layer.
- `english-phrase` owns phrase structure, clause structure, and realization.
- the nominal layer is **DP-first**
- NP survives only as the nominal core below D
- complements, modifiers, and adjuncts stay structurally distinct
- clauses are modeled as projections, not as string templates

## End-state vision

When this roadmap is complete, `english-phrase` should look like a compact X-bar-based realizer with:

- `DeterminerPhrase`
- `NominalPhrase`
- `AdjectivePhrase`
- `VerbPhrase`
- `PrepositionalPhrase`
- `AdverbPhrase`
- `Clause`
- `Sentence`

The current `NounPhrase` API should be understood as transitional. The future system should be conceptually DP-based from top to bottom.

## Phase 0: align terminology and documentation

### Goal

Make sure all future-facing design work treats the nominal layer as DP.

### Work

- document that current `NounPhrase` usage is temporary surface naming
- stop describing NP as the top nominal projection
- use DP terminology in new design notes and API sketches

### Deliverables

- DP-first docs
- no new roadmap material that treats NP as maximal

## Phase 1: introduce missing phrase projections

### Goal

Add the phrase types needed for a coherent X-bar inventory.

### Add

- `PrepositionalPhrase`
- `AdverbPhrase`
- `Coordination<T>`

### Why first

Before the nominal and clause layers can become structurally rich, the library needs the major projections that DP, VP, AdjP, and Clause will depend on.

### Target shapes

```rust
pub struct PrepositionalPhrase {
    head: Preposition,
    complement: PrepositionalComplement,
}

pub enum PrepositionalComplement {
    DeterminerPhrase(Box<DeterminerPhrase>),
    AdverbPhrase(Box<AdverbPhrase>),
    Clause(Box<NonFiniteClause>),
    Text(Text),
}

pub struct AdverbPhrase {
    head: Adverb,
    modifiers: Vec<AdverbModifier>,
    complements: Vec<AdverbComplement>,
}
```

## Phase 2: split DP from NP

### Goal

Replace the current single nominal object with an explicit DP-over-NP architecture.

### Structural rule

- **DP** is the external nominal phrase
- **NP** is the nominal core

### Add or refine

- determiner layer
- possessive layer
- quantifier layer
- numeral layer
- NP premodifiers
- NP complements
- NP postmodifiers
- relative clauses
- pronominal DPs

### Target shapes

```rust
pub struct DeterminerPhrase {
    determiner: Option<Determiner>,
    possessive: Option<Possessive>,
    quantifier: Option<Quantifier>,
    numeral: Option<Numeral>,
    nominal: NominalPhrase,
}

pub struct NominalPhrase {
    premodifiers: Vec<NominalPremodifier>,
    head: Noun,
    complements: Vec<NominalComplement>,
    postmodifiers: Vec<NominalPostmodifier>,
}
```

### Public API target

```rust
let dp = DeterminerPhrase::new("child")
    .determiner(Determiner::the())
    .quantifier(Quantifier::many())
    .modifier(AdjectivePhrase::new("small").comparative())
    .postmodifier(
        PrepositionalPhrase::new(
            "from",
            DeterminerPhrase::new("building").the().plural(),
        ),
    )
    .plural();
```

### Result

After this phase, the crate no longer treats “noun phrase” as the maximal nominal object. DP is the governing abstraction.

## Phase 3: complete AdjP and PP attachment structure

### Goal

Make adjectival and prepositional structure properly X-bar-like.

### Add or refine

- structured adjective modifiers
- PP complements to adjectives
- comparative complements
- infinitival complements to adjectives
- recursive PP attachment as postmodification

### Target shapes

```rust
pub struct AdjectivePhrase {
    degree: Degree,
    modifiers: Vec<AdjectiveModifier>,
    head: Adj,
    complements: Vec<AdjectiveComplement>,
}

pub enum AdjectiveComplement {
    PrepositionalPhrase(PrepositionalPhrase),
    NonFiniteClause(NonFiniteClause),
    Comparative(ComparativeComplement),
    DeterminerPhrase(Box<DeterminerPhrase>),
    Text(Text),
}
```

## Phase 4: complete VP structure

### Goal

Make the VP a proper projection rather than a tense/aspect wrapper around a verb.

### Add or refine

- lexical head
- particle position
- complement slots
- adjunct slots
- auxiliary projection above the lexical V
- finite vs non-finite verbal forms

### Target shapes

```rust
pub struct VerbPhrase {
    head: Verb,
    particle: Option<Particle>,
    complements: Vec<VerbComplement>,
    adjuncts: Vec<VerbAdjunct>,
    tense: Tense,
    aspect: Aspect,
    polarity: Polarity,
}
```

### Public API target

```rust
let vp = VerbPhrase::new("give")
    .past()
    .simple()
    .affirmative()
    .complement(
        VerbComplement::DeterminerPhrase(
            DeterminerPhrase::new("child").the(),
        )
    )
    .complement(
        VerbComplement::DeterminerPhrase(
            DeterminerPhrase::new("apple").an(),
        )
    );
```

## Phase 5: build clause projections

### Goal

Place the clause system clearly above VP and distinguish finite from non-finite projections.

### Add or refine

- finite clauses
- non-finite clauses
- relative clauses
- complement clauses
- clause adjuncts
- clause coordination

### Target shapes

```rust
pub struct Clause {
    subject: Option<DeterminerPhrase>,
    predicate: VerbPhrase,
    complements: Vec<ClauseComplement>,
    adjuncts: Vec<ClauseAdjunct>,
    clause_type: ClauseType,
}

pub struct NonFiniteClause {
    form: NonFiniteForm,
    predicate: VerbPhrase,
    subject: Option<DeterminerPhrase>,
    complements: Vec<ClauseComplement>,
}
```

### Finite vs non-finite clause note

A **finite clause** carries finiteness:

- `the child left`
- `the children have eaten`

A **non-finite clause** does not:

- `to leave`
- `leaving early`
- `having eaten`

These projections matter because non-finite clauses occur naturally as complements inside DP, AdjP, and PP.

## Phase 6: add relative and complement clause attachment

### Goal

Make clausal postmodification and clausal complementation first-class.

### Add or refine

- relative clause attachment inside DP
- clausal complements to adjectives
- clausal complements to prepositions
- clausal complements to verbs

### Public API target

```rust
let dp = DeterminerPhrase::new("child")
    .determiner(Determiner::the())
    .relative_clause(
        RelativeClause::who(
            Clause::new(
                DeterminerPhrase::gap(),
                VerbPhrase::new("wait").past().progressive().affirmative(),
            )
        )
    )
    .plural();
```

## Phase 7: sentence layer

### Goal

Keep orthography and punctuation above syntax.

### Add

- capitalization policy
- punctuation policy
- sentence-final terminal handling

### Public API target

```rust
let sentence = Clause::new(
    DeterminerPhrase::new("child").the().plural(),
    VerbPhrase::new("steal")
        .past()
        .simple()
        .affirmative()
        .complement(
            ClauseComplement::DeterminerPhrase(
                DeterminerPhrase::new("potato").count(7),
            )
        ),
)
.sentence()
.capitalize()
.period();
```

## Suggested implementation order

1. Add first-class PP and AdvP.
2. Split the nominal layer into DP and NP.
3. Expand AdjP and PP attachment.
4. Expand VP structure.
5. Add finite and non-finite clauses.
6. Add relative and complement clauses.
7. Add sentence-level orthography.

## End-state API sketch

```rust
let dp = DeterminerPhrase::new("child")
    .determiner(Determiner::the())
    .modifier(AdjectivePhrase::new("small").comparative())
    .postmodifier(
        PrepositionalPhrase::new(
            "from",
            DeterminerPhrase::new("building").the().plural(),
        ),
    )
    .plural();

let vp = VerbPhrase::new("eat")
    .present()
    .perfect()
    .negative()
    .agree_with(&dp);

let clause = Clause::new(dp, vp)
    .complement(
        ClauseComplement::DeterminerPhrase(
            DeterminerPhrase::new("potato").count(7),
        )
    );

let sentence = clause.sentence().capitalize().period();

assert_eq!(
    sentence.render(),
    "The smaller children have not eaten 7 potatoes."
);
```
