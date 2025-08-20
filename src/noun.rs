#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noun {
    pub head: String,
    pub modifier: Option<String>,   // words before the head
    pub complement: Option<String>, // words after the head
}

impl Noun {
    pub fn new(head: impl Into<String>) -> Self {
        Noun {
            head: head.into(),
            modifier: None,
            complement: None,
        }
    }

    pub fn with_specifier(mut self, pre: impl Into<String>) -> Self {
        self.modifier = Some(pre.into());
        self
    }

    pub fn with_complement(mut self, post: impl Into<String>) -> Self {
        self.complement = Some(post.into());
        self
    }
}

impl From<String> for Noun {
    fn from(s: String) -> Self {
        Noun {
            head: s,
            modifier: None,
            complement: None,
        }
    }
}
impl From<&String> for Noun {
    fn from(s: &String) -> Self {
        Noun {
            head: s.clone(),
            modifier: None,
            complement: None,
        }
    }
}

impl From<&str> for Noun {
    fn from(s: &str) -> Self {
        Noun {
            head: s.to_string(),
            modifier: None,
            complement: None,
        }
    }
}
impl From<&Noun> for Noun {
    fn from(s: &Noun) -> Self {
        s.clone()
    }
}
