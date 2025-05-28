use english::*;

use std::time::Instant;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel_derive::Steel;

fn plural(word: String) -> Option<String> {
    Some(EnglishCore::noun(&word, &Number::Plural))
}

fn main() {
    let mut vm = Engine::new();
    vm.register_fn("plural", plural);
    println!(
        "{:#?}",
        EnglishCore::verb(
            "eat",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        )
    );

    let noun1 = Noun {
        word: "man".to_string(),
        number: Number::Singular,
    };
    let verb1 = Verb {
        word: "eat".to_string(),
        tense: Tense::Past,
        person: Person::Third,
        form: Form::Finite,
    };
    println!("{:#?}", English::simple_sentence(&noun1, &noun1, &verb1));

    println!("{}", English::noun("thyridium", &Number::Plural));
    benchmark_verb();

    /*   loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim().to_lowercase();
        if command == "quit" {
            panic!("byebye")
        }

        match vm.run(command) {
            Ok(x) => {
                println!(" {x:#?} ")
            }
            Err(y) => {
                println!(" {y:#?} ")
            }
        }
    } */
}

pub fn benchmark_verb() {
    let words = ["tango", "dance", "ziknik"]; //"yak", "yandex", "zebra"
    let person = Person::Third;
    let number = Number::Singular;
    let tense = Tense::Present;
    let form = Form::Finite;

    let iterations = 1_000_000;

    let start = Instant::now();
    let mut result = String::new();

    for _ in 0..iterations {
        for &word in &words {
            result = English::verb(word, &person, &number, &tense, &form);
            //std::hint::black_box(&result); // Prevents compiler optimizations
        }
    }

    let duration = start.elapsed();
    println!("{result}");
    println!(
        "Benchmark completed in {:?} ({} total calls)",
        duration,
        iterations * words.len()
    );
}
