use english::*;

fn main() {
    println!("{:#?}", English::noun("cow", Number::Plural));
    println!("{:#?}", English::noun("blin", Number::Plural));
    println!("{:#?}", English::noun("milk", Number::Plural));
    println!("{:#?}", English::noun("Americanese", Number::Plural));
    println!("{:#?}", English::noun("you", Number::Plural));
}
