use std::{env, fs};

use mova::{interpreter::Value, runner::run};

fn main() {
    let args: Vec<String> = env::args().collect();
    let paths = &args[1..];

    paths.into_iter().for_each(|path| {
        let input = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file {path}: {e}");
                std::process::exit(1);
            }
        };

        match run(&input) {
            Ok(result) => {
                if let Some(value) = result {
                    match value {
                        Value::Reference(r) => {
                            let guard = r.read();
                            match &guard.value {
                                value => println!("{value:?}"),
                            }
                        }
                        _ => println!("{value:?}"),
                    }
                }
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    });
}
