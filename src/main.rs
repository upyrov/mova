#![cfg(not(target_arch = "wasm32"))]

use std::{env, fs};

use mova::runner::run;

fn main() {
    ctrlc::set_handler(move || std::process::exit(0)).expect("Error setting Ctrl-C handler");

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
            Ok(_) => {
                mova::runner::OUTPUT_BUFFER.with(|b| {
                    print!("{}", b.borrow());
                });
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    });
}
