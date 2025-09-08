use mova::*;

fn main() {
    let tokens = lexer::tokenize("1 + 2");
    let ast = parser::parse(tokens);
    let result = interpreter::evaluate(ast, None);
    println!("{:?}", result);
}
