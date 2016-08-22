pub mod parser;
pub mod ast;

fn main() {
    println!("Hello, world!");
}

#[test]
fn parse_graph_line() {
    assert_eq!(parser::parse_GraphLine("a => b => \nc;").unwrap(),
              vec![ast::GraphIdent::Identifier("a".to_string()),
                   ast::GraphIdent::Identifier("b".to_string()),
                   ast::GraphIdent::Identifier("c".to_string())]);
}
