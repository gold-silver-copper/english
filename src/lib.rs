pub use english_core::grammar::*;
mod nounsiki;
pub use nounsiki::*;
mod verbsiki;
pub use verbsiki::*;

pub struct English {}
impl English {
    pub fn noun(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => {
                return word.to_string();
            }
            Number::Plural => {
                if let Some(x) = get_plural(word) {
                    return x.to_string();
                } else {
                    return EnglishCore::noun(word, number);
                }
            }
        }
    }
}
