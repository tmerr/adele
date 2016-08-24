extern crate petgraph;

pub mod parser;
pub mod ast;

fn main() {
    println!("Hello, world!");
}

const TESTSOURCE: &'static str =
r#"

    systems gui model;

    type color = red | blue;
    type maybecolor = red | blue | neither;
    alias place_column = color * integer 0 7;
    alias game_state = color * array (array maybecolor 6) 7;
    alias game_over_state = maybecolor * game_state;

    msg gui place_disc place_column;
    msg model update_board game_state;
    msg model announce_game_over game_over_state;

    connect => place_disc => update_board => place_disc;
               place_disc => announce_game_over => disconnect;

"#;

#[test]
fn parse_graph_line() {
    assert_eq!(parser::parse_GraphLine("a => b => \nc;").unwrap(),
              vec![ast::GraphIdent::Identifier("a".to_string()),
                   ast::GraphIdent::Identifier("b".to_string()),
                   ast::GraphIdent::Identifier("c".to_string())]); 
}

#[test]
fn parse_source() {
    let root = parser::parse_Root(TESTSOURCE).unwrap();
    assert_eq!(root.systems, ast::SystemsDecl("gui".to_string(), "model".to_string()));
    assert_eq!(root.types.len(), 5);
    assert_eq!(root.messages.len(), 3);
    assert_eq!(root.graph.node_count(), 5);
    assert_eq!(root.graph.edge_count(), 5);
}
