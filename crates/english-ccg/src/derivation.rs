use std::ops::{Add, Range};

use english::{Animacy, Gender, Number, Person};

use crate::builders::{Conj, Modal, PrepRole, VerbFormKind};
use crate::cat::{bwd, can_bapply, can_bcomp, can_fapply, can_fcomp, fwd, type_raise, Cat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivTree {
    Leaf {
        surface: String,
        cat: Cat,
    },
    Inner {
        cat: Cat,
        rule: CombRule,
        left: Box<DerivTree>,
        right: Box<DerivTree>,
    },
    Gap {
        cat: Cat,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombRule {
    ForwardApplication,
    BackwardApplication,
    ForwardComposition,
    BackwardComposition,
    TypeRaising,
    Coordination,
}

#[derive(Debug, Clone)]
pub struct Ccg {
    pub(crate) atoms: Vec<Atom>,
    pub(crate) reduced: Option<Parse>,
    pub(crate) surface: String,
}

#[derive(Debug, Clone)]
pub(crate) enum Atom {
    Lex(Token),
    Opaque(OpaqueAtom),
}

#[derive(Debug, Clone)]
pub(crate) struct OpaqueAtom {
    pub cat: Cat,
    pub derivation: DerivTree,
    pub tokens: Vec<Token>,
    pub meta: ParseMeta,
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub surface: String,
    pub cat: Cat,
    pub kind: TokenKind,
    pub animacy: Option<Animacy>,
    pub agreement: Option<AgreementInfo>,
}

#[derive(Debug, Clone)]
pub(crate) enum TokenKind {
    Plain,
    Name,
    Noun { lemma: String },
    Pronoun,
    Verb { lemma: String, form: VerbFormKind },
    Modal { modal: Modal },
    Prep { _lemma: String, _role: PrepRole },
    Gap { original: Cat },
    Conj { _conj: Conj },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgreementInfo {
    pub person: Person,
    pub number: Number,
    pub gender: Gender,
}

#[derive(Debug, Clone)]
pub(crate) struct Parse {
    pub cat: Cat,
    pub derivation: DerivTree,
    pub meta: ParseMeta,
}

#[derive(Debug, Clone)]
pub(crate) struct ParseMeta {
    pub span: Range<usize>,
    pub np_agreement: Option<AgreementInfo>,
    pub subject_agreement: Option<AgreementInfo>,
    pub animacy: Option<Animacy>,
    pub subject_range: Option<Range<usize>>,
}

impl Default for ParseMeta {
    fn default() -> Self {
        Self {
            span: 0..0,
            np_agreement: None,
            subject_agreement: None,
            animacy: None,
            subject_range: None,
        }
    }
}

impl Ccg {
    pub(crate) fn from_token(token: Token) -> Self {
        let atoms = vec![Atom::Lex(token)];
        let reduced = parse_full(&atoms);
        let surface = surface_from_atoms(&atoms);
        Self {
            atoms,
            reduced,
            surface,
        }
    }

    pub(crate) fn from_opaque(atom: OpaqueAtom) -> Self {
        let atoms = vec![Atom::Opaque(atom)];
        let reduced = parse_full(&atoms);
        let surface = surface_from_atoms(&atoms);
        Self {
            atoms,
            reduced,
            surface,
        }
    }

    pub fn cat(&self) -> &Cat {
        &self
            .reduced
            .as_ref()
            .unwrap_or_else(|| panic!("{}", self.pending_diagnostic()))
            .cat
    }

    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub fn with_cat(self, cat: Cat) -> Self {
        if self.atoms.len() == 1 {
            let atoms: Vec<Atom> = self
                .atoms
                .into_iter()
                .map(|atom| match atom {
                    Atom::Lex(mut token) => {
                        token.cat = cat.clone();
                        if let TokenKind::Gap { original } = &token.kind {
                            token.cat = fwd(original.clone(), original.clone());
                        }
                        Atom::Lex(token)
                    }
                    Atom::Opaque(mut opaque) => {
                        opaque.cat = cat.clone();
                        override_tree_root_cat(&mut opaque.derivation, cat.clone());
                        Atom::Opaque(opaque)
                    }
                })
                .collect();
            let reduced = parse_full(&atoms);
            let surface = surface_from_atoms(&atoms);
            return Self {
                reduced,
                atoms,
                surface,
            };
        }

        let parse = self
            .reduced
            .unwrap_or_else(|| panic!("cannot override category of an unreduced derivation"));
        let mut derivation = parse.derivation.clone();
        override_tree_root_cat(&mut derivation, cat.clone());
        let tokens = flatten_tokens(&self.atoms);
        let opaque = OpaqueAtom {
            cat: cat.clone(),
            derivation: derivation.clone(),
            tokens: tokens.clone(),
            meta: ParseMeta {
                span: 0..tokens.len(),
                np_agreement: parse.meta.np_agreement,
                subject_agreement: parse.meta.subject_agreement,
                animacy: parse.meta.animacy,
                subject_range: parse.meta.subject_range,
            },
        };
        Self::from_opaque(opaque)
    }

    pub fn derivation(&self) -> &DerivTree {
        &self
            .reduced
            .as_ref()
            .unwrap_or_else(|| panic!("{}", self.pending_diagnostic()))
            .derivation
    }

    pub fn animate(mut self) -> Self {
        self.set_animacy(Animacy::Animate);
        self
    }

    pub fn inanimate(mut self) -> Self {
        self.set_animacy(Animacy::Inanimate);
        self
    }

    pub(crate) fn subject_agreement(&self) -> Option<AgreementInfo> {
        self.reduced
            .as_ref()
            .and_then(|parse| parse.meta.subject_agreement)
    }

    pub(crate) fn subject_range(&self) -> Option<Range<usize>> {
        self.reduced
            .as_ref()
            .and_then(|parse| parse.meta.subject_range.clone())
    }

    pub(crate) fn flattened_tokens(&self) -> Vec<Token> {
        flatten_tokens(&self.atoms)
    }

    pub(crate) fn reduced_parse(&self) -> Option<&Parse> {
        self.reduced.as_ref()
    }

    fn set_animacy(&mut self, animacy: Animacy) {
        for atom in &mut self.atoms {
            match atom {
                Atom::Lex(token) => token.animacy = Some(animacy),
                Atom::Opaque(opaque) => {
                    opaque.meta.animacy = Some(animacy);
                    for token in &mut opaque.tokens {
                        token.animacy = Some(animacy);
                    }
                }
            }
        }
        self.reduced = parse_full(&self.atoms);
        self.surface = surface_from_atoms(&self.atoms);
    }

    fn pending_diagnostic(&self) -> String {
        let cats = self
            .atoms
            .iter()
            .map(Atom::cat)
            .map(|cat| cat.to_notation())
            .collect::<Vec<_>>()
            .join(" + ");
        format!("unreduced CCG sequence: {cats}")
    }
}

impl Add for Ccg {
    type Output = Ccg;

    fn add(self, rhs: Ccg) -> Self::Output {
        let mut atoms = self.atoms;
        atoms.extend(rhs.atoms);
        let reduced = parse_full(&atoms);
        let surface = surface_from_atoms(&atoms);
        Ccg {
            atoms,
            reduced,
            surface,
        }
    }
}

impl Atom {
    pub(crate) fn cat(&self) -> &Cat {
        match self {
            Self::Lex(token) => &token.cat,
            Self::Opaque(opaque) => &opaque.cat,
        }
    }

    fn base_parse(&self, span: Range<usize>) -> Parse {
        match self {
            Self::Lex(token) => match &token.kind {
                TokenKind::Gap { original } => Parse {
                    cat: token.cat.clone(),
                    derivation: DerivTree::Gap {
                        cat: original.clone(),
                    },
                    meta: ParseMeta {
                        span,
                        ..ParseMeta::default()
                    },
                },
                _ => Parse {
                    cat: token.cat.clone(),
                    derivation: DerivTree::Leaf {
                        surface: token.surface.clone(),
                        cat: token.cat.clone(),
                    },
                    meta: ParseMeta {
                        span,
                        np_agreement: lexical_np_agreement(token),
                        subject_agreement: None,
                        animacy: lexical_animacy(token),
                        subject_range: None,
                    },
                },
            },
            Self::Opaque(opaque) => {
                let mut meta = opaque.meta.clone();
                meta.span = span;
                Parse {
                    cat: opaque.cat.clone(),
                    derivation: opaque.derivation.clone(),
                    meta,
                }
            }
        }
    }
}

fn flatten_tokens(atoms: &[Atom]) -> Vec<Token> {
    let mut tokens = Vec::new();
    for atom in atoms {
        match atom {
            Atom::Lex(token) => tokens.push(token.clone()),
            Atom::Opaque(opaque) => tokens.extend(opaque.tokens.clone()),
        }
    }
    tokens
}

fn surface_from_atoms(atoms: &[Atom]) -> String {
    flatten_tokens(atoms)
        .into_iter()
        .filter(|token| !matches!(token.kind, TokenKind::Gap { .. }))
        .map(|token| token.surface)
        .collect::<Vec<_>>()
        .join(" ")
}

fn lexical_np_agreement(token: &Token) -> Option<AgreementInfo> {
    match &token.kind {
        TokenKind::Name | TokenKind::Pronoun | TokenKind::Noun { .. } => token.agreement,
        _ => None,
    }
}

fn lexical_animacy(token: &Token) -> Option<Animacy> {
    match token.kind {
        TokenKind::Name | TokenKind::Pronoun | TokenKind::Noun { .. } => token.animacy,
        _ => None,
    }
}

pub(crate) fn parse_full(atoms: &[Atom]) -> Option<Parse> {
    if atoms.is_empty() {
        return None;
    }

    let spans = atom_token_spans(atoms);
    let n = atoms.len();
    let mut chart = vec![vec![Vec::<Parse>::new(); n + 1]; n];

    for i in 0..n {
        let span = spans[i].clone();
        let base = atoms[i].base_parse(span);
        push_parse(&mut chart[i][i + 1], base.clone());
        if let Some(raised) = maybe_type_raise(&base) {
            push_parse(&mut chart[i][i + 1], raised);
        }
    }

    for width in 2..=n {
        for start in 0..=n - width {
            let end = start + width;
            for split in start + 1..end {
                let left_parses = chart[start][split].clone();
                let right_parses = chart[split][end].clone();
                for left in &left_parses {
                    for right in &right_parses {
                        if let Some(parse) = combine_forward_application(left, right) {
                            push_parse(&mut chart[start][end], parse);
                        }
                        if let Some(parse) = combine_backward_application(left, right) {
                            push_parse(&mut chart[start][end], parse);
                        }
                        if let Some(parse) = combine_forward_composition(left, right) {
                            push_parse(&mut chart[start][end], parse);
                        }
                        if let Some(parse) = combine_backward_composition(left, right) {
                            push_parse(&mut chart[start][end], parse);
                        }
                    }
                }

                let current = chart[start][end].clone();
                for parse in current {
                    if let Some(raised) = maybe_type_raise(&parse) {
                        push_parse(&mut chart[start][end], raised);
                    }
                }
            }
        }
    }

    chart[0][n].first().cloned()
}

fn atom_token_spans(atoms: &[Atom]) -> Vec<Range<usize>> {
    let mut next = 0;
    atoms
        .iter()
        .map(|atom| {
            let width = match atom {
                Atom::Lex(_) => 1,
                Atom::Opaque(opaque) => opaque.tokens.len(),
            };
            let span = next..next + width;
            next += width;
            span
        })
        .collect()
}

fn push_parse(cell: &mut Vec<Parse>, parse: Parse) {
    if !cell.iter().any(|existing| existing.cat == parse.cat) {
        cell.push(parse);
    }
}

fn maybe_type_raise(parse: &Parse) -> Option<Parse> {
    let cat = type_raise(&parse.cat)?;
    Some(Parse {
        cat: cat.clone(),
        derivation: DerivTree::Inner {
            cat,
            rule: CombRule::TypeRaising,
            left: Box::new(parse.derivation.clone()),
            right: Box::new(DerivTree::Gap {
                cat: bwd(Cat::S, Cat::NP),
            }),
        },
        meta: ParseMeta {
            span: parse.meta.span.clone(),
            np_agreement: None,
            subject_agreement: parse.meta.np_agreement,
            animacy: parse.meta.animacy,
            subject_range: Some(parse.meta.span.clone()),
        },
    })
}

fn combine_forward_application(left: &Parse, right: &Parse) -> Option<Parse> {
    let cat = can_fapply(&left.cat, &right.cat)?;
    Some(Parse {
        cat: cat.clone(),
        derivation: DerivTree::Inner {
            cat: cat.clone(),
            rule: CombRule::ForwardApplication,
            left: Box::new(left.derivation.clone()),
            right: Box::new(right.derivation.clone()),
        },
        meta: meta_for_forward_application(&cat, left, right),
    })
}

fn combine_backward_application(left: &Parse, right: &Parse) -> Option<Parse> {
    let cat = can_bapply(&left.cat, &right.cat)?;
    Some(Parse {
        cat: cat.clone(),
        derivation: DerivTree::Inner {
            cat: cat.clone(),
            rule: CombRule::BackwardApplication,
            left: Box::new(left.derivation.clone()),
            right: Box::new(right.derivation.clone()),
        },
        meta: meta_for_backward_application(&cat, left, right),
    })
}

fn combine_forward_composition(left: &Parse, right: &Parse) -> Option<Parse> {
    let cat = can_fcomp(&left.cat, &right.cat)?;
    Some(Parse {
        cat: cat.clone(),
        derivation: DerivTree::Inner {
            cat: cat.clone(),
            rule: CombRule::ForwardComposition,
            left: Box::new(left.derivation.clone()),
            right: Box::new(right.derivation.clone()),
        },
        meta: meta_for_composition(&cat, left, right),
    })
}

fn combine_backward_composition(left: &Parse, right: &Parse) -> Option<Parse> {
    let cat = can_bcomp(&left.cat, &right.cat)?;
    Some(Parse {
        cat: cat.clone(),
        derivation: DerivTree::Inner {
            cat: cat.clone(),
            rule: CombRule::BackwardComposition,
            left: Box::new(left.derivation.clone()),
            right: Box::new(right.derivation.clone()),
        },
        meta: meta_for_composition(&cat, left, right),
    })
}

fn meta_for_forward_application(cat: &Cat, left: &Parse, right: &Parse) -> ParseMeta {
    let mut meta = ParseMeta {
        span: left.meta.span.start..right.meta.span.end,
        ..ParseMeta::default()
    };

    if matches!(cat, Cat::NP | Cat::N) {
        meta.np_agreement = right.meta.np_agreement.or(left.meta.np_agreement);
        meta.animacy = right.meta.animacy.or(left.meta.animacy);
    }

    if *cat == Cat::S {
        meta.subject_agreement = left.meta.subject_agreement.or(right.meta.subject_agreement);
        meta.subject_range = left
            .meta
            .subject_range
            .clone()
            .or(right.meta.subject_range.clone());
    } else {
        meta.subject_agreement = left.meta.subject_agreement.or(right.meta.subject_agreement);
        meta.subject_range = left
            .meta
            .subject_range
            .clone()
            .or(right.meta.subject_range.clone());
    }

    meta
}

fn meta_for_backward_application(cat: &Cat, left: &Parse, right: &Parse) -> ParseMeta {
    let mut meta = ParseMeta {
        span: left.meta.span.start..right.meta.span.end,
        ..ParseMeta::default()
    };

    if matches!(cat, Cat::NP | Cat::N) {
        meta.np_agreement = left.meta.np_agreement.or(right.meta.np_agreement);
        meta.animacy = left.meta.animacy.or(right.meta.animacy);
    }

    if *cat == Cat::S && left.meta.np_agreement.is_some() {
        meta.subject_agreement = left.meta.np_agreement;
        meta.subject_range = Some(left.meta.span.clone());
    } else {
        meta.subject_agreement = right.meta.subject_agreement.or(left.meta.subject_agreement);
        meta.subject_range = right
            .meta
            .subject_range
            .clone()
            .or(left.meta.subject_range.clone());
    }

    meta
}

fn meta_for_composition(cat: &Cat, left: &Parse, right: &Parse) -> ParseMeta {
    let mut meta = ParseMeta {
        span: left.meta.span.start..right.meta.span.end,
        ..ParseMeta::default()
    };

    if matches!(cat, Cat::NP | Cat::N) {
        meta.np_agreement = left.meta.np_agreement.or(right.meta.np_agreement);
        meta.animacy = left.meta.animacy.or(right.meta.animacy);
    }

    meta.subject_agreement = left.meta.subject_agreement.or(right.meta.subject_agreement);
    meta.subject_range = left
        .meta
        .subject_range
        .clone()
        .or(right.meta.subject_range.clone());

    meta
}

fn override_tree_root_cat(tree: &mut DerivTree, cat: Cat) {
    match tree {
        DerivTree::Leaf { cat: root, .. } => *root = cat,
        DerivTree::Inner { cat: root, .. } => *root = cat,
        DerivTree::Gap { cat: root } => *root = cat,
    }
}

pub(crate) fn token(
    surface: impl Into<String>,
    cat: Cat,
    kind: TokenKind,
    animacy: Option<Animacy>,
    agreement: Option<AgreementInfo>,
) -> Ccg {
    Ccg::from_token(Token {
        surface: surface.into(),
        cat,
        kind,
        animacy,
        agreement,
    })
}

pub(crate) fn opaque_atom(
    cat: Cat,
    derivation: DerivTree,
    tokens: Vec<Token>,
    meta: ParseMeta,
) -> Ccg {
    Ccg::from_opaque(OpaqueAtom {
        cat,
        derivation,
        tokens,
        meta,
    })
}

pub(crate) fn assemble(atoms: Vec<Atom>, reduced: Option<Parse>) -> Ccg {
    let surface = surface_from_atoms(&atoms);
    Ccg {
        atoms,
        reduced,
        surface,
    }
}
