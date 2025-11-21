use std::{env, fs};

use mova::runner::run;

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
                if let Some(data) = result {
                    println!("{data:?}");
                }
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    });
}
