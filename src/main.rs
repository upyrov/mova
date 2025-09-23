use std::{env, fs};

use mova::runner::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    let paths = &args[1..];

    paths.into_iter().for_each(|path| {
        let input = fs::read_to_string(path).expect("Unable to read file");
        let result = run(&input);
        println!("{:?}", result);
    });
}
