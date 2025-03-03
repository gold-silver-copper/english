use english::*;

fn main() {
    let numer = Number::Plural;
    println!("{:#?}", English::noun("cow", &numer));
    println!("{:#?}", English::noun("blin", &numer));
    println!("{:#?}", English::noun("milk", &numer));
    println!("{:#?}", English::noun("Americanese", &numer));
    println!("{:#?}", English::noun("you", &numer));
    println!("{:#?}", English::noun("man", &numer));
    println!("{:#?}", English::noun("wereman", &numer));
    println!(
        "{:#?}",
        English::verb("eat", &Person::Third, &Number::Singular, &Tense::SimplePast)
    );
}
