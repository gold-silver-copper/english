use crate::builders::Animacy;
use crate::cat::{bwd, fwd, Cat};

pub trait Lexicon {
    fn lookup(&self, word: &str) -> Option<LexEntry>;
}

#[derive(Debug, Clone)]
pub struct LexEntry {
    pub surface: String,
    pub cat: Cat,
    pub animacy: Animacy,
    pub irregulars: IrregularForms,
}

#[derive(Debug, Clone, Default)]
pub struct IrregularForms {
    pub past: Option<String>,
    pub perfective: Option<String>,
    pub past_participle: Option<String>,
    pub present_participle: Option<String>,
    pub plural: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BuiltinLexicon;

impl Lexicon for BuiltinLexicon {
    fn lookup(&self, word: &str) -> Option<LexEntry> {
        Some(match word {
            "arrive" | "go" | "run" => LexEntry {
                surface: word.to_string(),
                cat: bwd(Cat::S, Cat::NP),
                animacy: Animacy::Inanimate,
                irregulars: irregulars(word),
            },
            "say" => LexEntry {
                surface: word.to_string(),
                cat: fwd(bwd(Cat::S, Cat::NP), Cat::S),
                animacy: Animacy::Inanimate,
                irregulars: irregulars(word),
            },
            "promise" => LexEntry {
                surface: word.to_string(),
                cat: fwd(bwd(Cat::S, Cat::NP), bwd(Cat::S, Cat::NP)),
                animacy: Animacy::Inanimate,
                irregulars: irregulars(word),
            },
            "break" | "trust" | "inspect" | "damage" | "repair" => LexEntry {
                surface: word.to_string(),
                cat: fwd(bwd(Cat::S, Cat::NP), Cat::NP),
                animacy: Animacy::Inanimate,
                irregulars: irregulars(word),
            },
            "engineer" => LexEntry {
                surface: word.to_string(),
                cat: Cat::N,
                animacy: Animacy::Animate,
                irregulars: irregulars(word),
            },
            "Alice" | "Jordan" => LexEntry {
                surface: word.to_string(),
                cat: Cat::NP,
                animacy: Animacy::Animate,
                irregulars: irregulars(word),
            },
            _ => LexEntry {
                surface: word.to_string(),
                cat: Cat::N,
                animacy: Animacy::Inanimate,
                irregulars: irregulars(word),
            },
        })
    }
}

pub(crate) fn builtin() -> BuiltinLexicon {
    BuiltinLexicon
}

pub(crate) fn verb_cat(word: &str) -> Cat {
    builtin()
        .lookup(word)
        .map(|entry| entry.cat)
        .unwrap_or_else(|| fwd(bwd(Cat::S, Cat::NP), Cat::NP))
}

pub(crate) fn noun_animacy(word: &str) -> Animacy {
    builtin()
        .lookup(word)
        .map(|entry| entry.animacy)
        .unwrap_or(Animacy::Inanimate)
}

fn irregulars(word: &str) -> IrregularForms {
    match word {
        "go" => IrregularForms {
            past: Some("went".to_string()),
            perfective: Some("gone".to_string()),
            past_participle: Some("gone".to_string()),
            present_participle: Some("going".to_string()),
            plural: None,
        },
        "break" => IrregularForms {
            past: Some("broke".to_string()),
            perfective: Some("broken".to_string()),
            past_participle: Some("broken".to_string()),
            present_participle: Some("breaking".to_string()),
            plural: None,
        },
        "run" => IrregularForms {
            past: Some("ran".to_string()),
            perfective: Some("run".to_string()),
            past_participle: Some("run".to_string()),
            present_participle: Some("running".to_string()),
            plural: None,
        },
        _ => IrregularForms::default(),
    }
}
