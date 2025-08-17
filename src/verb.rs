#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    pub head: String,             // "pick"
    pub particle: Option<String>, // "up"
}

impl Verb {
    /// Create a new verb with just the head.
    pub fn new(head: impl Into<String>) -> Self {
        Verb {
            head: head.into(),
            particle: None,
        }
    }

    /// Set the particle of a phrasal verb.
    pub fn with_particle(mut self, particle: impl Into<String>) -> Self {
        self.particle = Some(particle.into());
        self
    }
}

impl From<String> for Verb {
    fn from(s: String) -> Self {
        Verb {
            head: s,
            particle: None,
        }
    }
}

impl From<&String> for Verb {
    fn from(s: &String) -> Self {
        Verb {
            head: s.clone(),
            particle: None,
        }
    }
}

impl From<&str> for Verb {
    fn from(s: &str) -> Self {
        Verb {
            head: s.to_string(),
            particle: None,
        }
    }
}

impl From<&Verb> for Verb {
    fn from(s: &Verb) -> Self {
        s.clone()
    }
}
