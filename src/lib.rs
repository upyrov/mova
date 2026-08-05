pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod runner;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
