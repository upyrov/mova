use mova::*;

fn main() {
    let tokens = lexer::tokenize("fn add(q){q + 5} add(4)");
    let ast = parser::parse(tokens);
    let result = interpreter::evaluate(ast, None);
    println!("{:?}", result);
}
