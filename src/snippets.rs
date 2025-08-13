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

use crate::*;

impl English {
    pub fn simple_sentence(object: &Noun, subject: &Noun, verb: &Verb) -> String {
        let verb_str = English::verb(
            &verb.word,
            &verb.person,
            &subject.number,
            &verb.tense,
            &verb.form,
        );

        format!("{} {} {}.", subject, verb_str, object)
    }
}
