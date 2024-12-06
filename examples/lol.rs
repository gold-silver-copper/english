use english::*;

fn main() {
    println!("{:#?}", English::noun("cow", Number::Singular));
    println!("{:#?}", English::noun("cow", Number::Plural));
    println!("{:#?}", English::noun("milk", Number::Singular));
    println!("{:#?}", English::noun("milk", Number::Plural));
    println!("{:#?}", English::noun("Americanese", Number::Singular));
    println!("{:#?}", English::noun("Americanese", Number::Plural));
}
