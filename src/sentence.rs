use crate::grammar::*;

impl English {
    pub fn simple_sentence(object: &Noun, subject: &Noun, verb: &Verb) -> String {
        let obj_str = English::noun(&object.word, &object.number);
        let subj_str = English::noun(&subject.word, &subject.number);
        let verb_str = English::verb(
            &verb.word,
            &verb.person,
            &object.number,
            &verb.tense,
            &verb.form,
        );

        format!("{} {} {}.", obj_str, verb_str, subj_str)
    }
}
