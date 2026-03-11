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
