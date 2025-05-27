use english::*;
use std::io::{self, Write};
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
    println!(
        "{:#?}",
        EnglishCore::simple_sentence(&noun1, &noun1, &verb1)
    );

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
