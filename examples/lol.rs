use english::*;
use std::io::{self, Write};
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel_derive::Steel;

fn plural(word: String) -> Option<String> {
    let form = NounFormSpec {
        number: Number::Plural,
        case: Case::Nominative, // Usually plural nouns default to nominative case
        gender: None,
    };
    Some(English::noun(&word, &form))
}

fn main() {
    let mut vm = Engine::new();
    vm.register_fn("plural", plural);
    let form = VerbFormSpec {
        form: VerbForm::Finite,
        tense: Some(Tense::Past),
        mood: Some(Mood::Indicative),
        number: Number::Singular,
        person: Person::Third,
    };

    println!("{:#?}", English::verb("eat", &form));

    /* loop {
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
