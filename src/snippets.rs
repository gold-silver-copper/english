#[derive(Debug, PartialEq, Clone)]
pub struct Noun {
    pub word: String,
    pub number: Number,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Verb {
    pub word: String,
    pub person: Person,
    pub tense: Tense,
    pub form: Form,
}

impl fmt::Display for Noun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", English::noun(&self.word, &self.number))
    }
}

/*
impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            English::verb(&self.word, &self.person, &self.tense, &self.form)
        )
    }
}

*/
