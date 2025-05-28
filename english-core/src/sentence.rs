use crate::grammar::*;
use crate::EnglishCore;
impl EnglishCore {
    pub fn simple_sentence(object: &Noun, subject: &Noun, verb: &Verb) -> String {
        let obj_str = EnglishCore::noun(&object.word, &object.number);
        let subj_str = EnglishCore::noun(&subject.word, &subject.number);
        let verb_str = EnglishCore::verb(
            &verb.word,
            &verb.person,
            &object.number,
            &verb.tense,
            &verb.form,
        );

        format!("{} {} {}.", obj_str, verb_str, subj_str)
    }
}
