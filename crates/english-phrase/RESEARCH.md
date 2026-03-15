# `english-phrase` research notes

## Goal

This document collects the sources most useful for developing `english-phrase` on a **strictly X-bar, DP-first foundation**.

The governing assumptions are:

- syntax should be modeled as **headed projections**
- **DP** is the maximal nominal projection
- **NP** is only the nominal projection inside DP
- phrase and clause generation should proceed from structured syntax to rendered text

This means the library should be designed around projections such as:

- `DeterminerPhrase`
- `NominalPhrase`
- `AdjectivePhrase`
- `VerbPhrase`
- `PrepositionalPhrase`
- `AdverbPhrase`
- `Clause`
- `Sentence`

The current crate still exposes `NounPhrase` in code, but these notes treat that name as temporary surface terminology. The underlying model should be DP-first.

## High-level conclusion

`english-phrase` should be grounded in:

1. **X-bar phrase structure** for the core architecture
2. **Abney’s DP hypothesis** for the nominal layer
3. **English descriptive grammars** to determine which structures English actually allows
4. **constituency-oriented corpora** to guide examples and testing

The central architectural consequence is:

- every major phrase should have a head
- complements should be structurally distinct from adjuncts
- specifier positions should exist where the theory requires them
- the nominal layer should be split into **DP over NP**
- clause structure should be modeled as a projection above the VP, not as formatted text around a verb

## Core commitments

### 1. DP over NP

For this project:

- **DP** is the full nominal phrase
- **NP** is the nominal core below D
- determiners, possessives, quantifiers, pronouns, and related nominal material belong in the DP layer

That implies an internal shape like:

```rust
pub struct DeterminerPhrase {
    determiner: Option<Determiner>,
    nominal: NominalPhrase,
}

pub struct NominalPhrase {
    head: Noun,
    complements: Vec<NominalComplement>,
    postmodifiers: Vec<NominalPostmodifier>,
}
```

### 2. Headed phrase structure

Every phrase should be modeled as a projection of a head:

- DP projects from D
- NP projects from N
- VP projects from V
- PP projects from P
- AdjP projects from A
- AdvP projects from Adv

This matters because it keeps us from collapsing everything into “base word plus strings.”

### 3. Specifier / complement / adjunct distinction

This is one of the most important X-bar distinctions for the library.

- **specifier**: structurally special dependent
- **complement**: selected or structurally close dependent to the head
- **adjunct**: optional modifier attached higher in the projection

If the crate gets this distinction right, later features will remain compositional instead of becoming special cases.

### 4. Clause structure above phrase structure

The clause should be modeled as a projection above the VP.

For English generation purposes, the library needs at least:

- finite clauses
- non-finite clauses
- relative clauses
- complement clauses

It is better to think in terms of clause projections than in terms of sentence templates.

## Best source stack

If you want the most useful source stack for this project, use:

1. **Carnie and X-bar teaching materials** for projection structure
2. **Abney** for DP
3. **CGEL and Quirk** for English descriptive detail
4. **Cambridge Grammar Today** for open examples
5. **Penn Treebank** for constituency inventories and testing

That stack is enough to guide the architecture without pulling the project into non-X-bar frameworks.

## Primary sources

### 1. Andrew Carnie

Use Carnie as the main general source for X-bar syntax and phrase structure.

Sources:

- [Constituent Structure, chapter “X-bar Theory”](https://academic.oup.com/book/48037/chapter-abstract/421291005)
- [Carnie textbook/workbook page](https://www.andrewcarnie.org/copy-of-textbook4e)
- [The Syntax Workbook](https://www.wiley-vch.de/en/areas-interest/humanities-social-sciences/linguistics-12lg/theoretical-linguistics-12lg1/syntax-12lg11/the-syntax-workbook-978-1-119-56929-9)

Why it matters:

- best single starting point for phrase-structure architecture
- especially useful for heads, complements, specifiers, adjuncts, and clause layering

### 2. Abney and the DP hypothesis

Use Abney as the central source for the nominal layer.

Sources:

- [The English Noun Phrase in Its Sentential Aspect](https://mitwpl.mit.edu/catalog/abne01)
- [CORE mirror / metadata for Abney 1987](https://core.ac.uk/display/24457458)
- [Glottolog metadata entry](https://glottolog.org/resource/reference/id/29190)

Why it matters:

- it provides the clearest theoretical basis for replacing NP as the maximal nominal projection
- it is directly relevant to determiners, possessives, quantifiers, pronouns, and nominal structure

Practical consequence:

- the future crate should expose or internally realize **DeterminerPhrase** as the top nominal object

### 3. UBC X-bar notes

Use these as a quick implementation-oriented refresher.

Source:

- [UBC X-bar Theory wiki](https://wiki.ubc.ca/Course%3ALING300/X-bar_Theory)

Why it matters:

- concise explanation of phrase projection
- helpful when translating theory into concrete data structures

### 4. SDSU notes on complements, specifiers, and adjuncts

Use these to sharpen the complement vs adjunct distinction.

Source:

- [Complements, Specifiers, and Adjuncts](https://gawron.sdsu.edu/syntax/new_midterm/adj_comp_spec.htm)

Why it matters:

- this distinction drives the shape of DP, VP, PP, AdjP, and Clause APIs

### 5. Phrase inventory slides for nominal and modifier projections

Use these for quick cross-category comparison.

Source:

- [General structure of NP, AdjP, AdvP, and PP](https://dept.english.wisc.edu/rfyoung/324/NPAdjPAdvPandPP.pdf)

Important note:

- these materials still use **NP** in the traditional way
- for this project, read them through a **DP-first** lens: NP is the nominal core, not the maximal nominal projection

## Descriptive English references

### 6. CGEL

Use CGEL as the highest-quality descriptive grammar when English structure becomes ambiguous.

- Bibliographic record: [Huddleston & Pullum 2002](https://grambank.clld.org/sources/gHuddlestonPullumEnglish)
- Retail metadata: [Barnes & Noble listing](https://www.barnesandnoble.com/w/the-cambridge-grammar-of-the-english-language-rodney-huddleston/1100045023)

Why it matters:

- strong source for English clause and phrase distinctions
- especially useful when deciding whether something is a complement, modifier, postmodifier, or clause-level dependent

### 7. Quirk et al.

Use Quirk as a secondary descriptive reference.

- Open Library record: [A Comprehensive Grammar of the English Language](https://openlibrary.org/books/OL18313957M/A_Comprehensive_grammar_of_the_English_language)
- WorldCat record: [A Comprehensive Grammar of the English Language](https://search.worldcat.org/title/A-Comprehensive-grammar-of-the-English-language/oclc/11533395)

Why it matters:

- broad English coverage
- helpful when comparing traditional phrase descriptions with the X-bar analysis

### 8. Cambridge English Grammar Today

Use this as the most implementation-friendly open source for examples and quick checks.

Relevant pages:

- [Noun phrases](https://dictionary.cambridge.org/us/grammar/british-grammar/phrases)
- [Noun phrases: noun phrases and verbs](https://dictionary.cambridge.org/us/grammar/british-grammar/noun-phrases-noun-phrases-and-verbs)
- [Adjective phrases](https://dictionary.cambridge.org/us/grammar/british-grammar/adjective-phrases)
- [Adjective phrases: functions](https://dictionary.cambridge.org/de/grammatik/britisch-grammatik/adjective-phrases-functions)
- [Adjective phrases: position](https://dictionary.cambridge.org/grammar/british-grammar/adjective-phrases-position%23adjective-phrases-position__153)
- [Verb phrases](https://dictionary.cambridge.org/us/grammar/british-grammar/verb-phrases)
- [Prepositional phrases](https://dictionary.cambridge.org/pt/gramatica/gramatica-britanica/prepositional-phrases)
- [Prepositions and particles](https://dictionary.cambridge.org/de/grammatik/british-grammar/prepositions-and-particles)
- [Adverb phrases](https://dictionary.cambridge.org/us/grammar/british-grammar/adverb-)
- [Clause types](https://dictionary.cambridge.org/us/grammar/british-grammar/clause-types)

Why it matters:

- open and practical
- good source for example sentences and API test cases

## Constituency and testing reference

### 9. Penn Treebank

Use PTB for practical constituency inventories and future testing.

Sources:

- [Building a Large Annotated Corpus of English: The Penn Treebank](https://aclanthology.org/J93-2004/)
- [Bracketing Guidelines for Treebank II Style Penn Treebank Project](https://repository.upenn.edu/handle/20.500.14332/6998)

Why it matters:

- useful source of concrete phrase and clause structures
- good for coordination, attachment, and relative-clause examples

Important caveat:

- PTB annotation still uses `NP`
- that is a treebank convention, not a reason to abandon a DP-first internal model

## What the phrase system should become

### Determiner phrase

The top nominal object should be DP, not NP.

Future internal shape:

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

### Adjective phrase

An adjective phrase should keep:

- degree
- modifiers
- head adjective
- complements

Future shape:

```rust
pub struct AdjectivePhrase {
    degree: Degree,
    modifiers: Vec<AdjectiveModifier>,
    head: Adj,
    complements: Vec<AdjectiveComplement>,
}
```

### Verb phrase

A VP should be modeled as a head plus its structural dependents:

- head verb
- particles where relevant
- complements
- adjuncts
- auxiliary projection above the lexical verb

Future shape:

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

### Prepositional phrase

A PP should be first-class and typed:

```rust
pub struct PrepositionalPhrase {
    head: Preposition,
    complement: PrepositionalComplement,
}
```

### Adverb phrase

An AdvP should eventually be its own phrase type, not free text:

```rust
pub struct AdverbPhrase {
    head: Adverb,
    modifiers: Vec<AdverbModifier>,
    complements: Vec<AdverbComplement>,
}
```

### Clause

Clause structure should sit above the VP and distinguish finite from non-finite projections.

At minimum, the library should eventually model:

- finite clauses
- non-finite clauses
- relative clauses
- complement clauses

Future shape:

```rust
pub struct Clause {
    subject: Option<DeterminerPhrase>,
    predicate: VerbPhrase,
    complements: Vec<ClauseComplement>,
    adjuncts: Vec<ClauseAdjunct>,
    clause_type: ClauseType,
}
```

## Future API direction

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

assert_eq!(clause.render(), "the smaller children have not eaten 7 potatoes");
```

## Recommended implementation priorities

1. Treat the nominal layer as DP everywhere in future design work.
2. Add first-class `PrepositionalPhrase` and `AdverbPhrase`.
3. Split the current nominal object into DP and NP/nominal core.
4. Expand clause structure as a projection above VP.
5. Add relative and complement clause support.
6. Add coordination once the major projections are stable.
