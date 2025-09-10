use mova::*;

fn main() {
    let tokens = lexer::tokenize("fn add(){let q = 1 + 2 q + 5}");
    let ast = parser::parse(tokens);
    let result = interpreter::evaluate(ast, None);
    println!("{:?}", result);
}
