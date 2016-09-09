extern crate petgraph;
extern crate lalrpop_util;

pub mod parser;
pub mod ast;
pub mod intermediate;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use parser;
    use ast;

    #[test]
    fn parse_graph_line() {
        let line = parser::parse_GraphLine("a => b => \nc;").unwrap();
        assert_eq!(line[0].node, ast::GraphIdent::Identifier("a".to_string()));
        assert_eq!(line[1].node, ast::GraphIdent::Identifier("b".to_string()));
        assert_eq!(line[2].node, ast::GraphIdent::Identifier("c".to_string()));
        assert_eq!(line.len(), 3);
    }

    #[test]
    fn example_pass() {
        let source = r#"
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
        let root = parser::parse_Root(source).unwrap();
        ast::validate(&root).unwrap();
        assert_eq!(root.systems.0.node, "gui".to_string());
        assert_eq!(root.systems.1.node, "model".to_string());
        assert_eq!(root.types.len(), 5);
        assert_eq!(root.messages.len(), 3);
        assert_eq!(root.graph.node_count(), 5);
        assert_eq!(root.graph.edge_count(), 5);
    }

    #[test]
    fn connect_ident_disconnect_pass() {
        let source = r#"
            systems gui model;
            msg model istrue bool;
            connect => istrue => disconnect;
        "#;
        let root = parser::parse_Root(source).unwrap();
    }


    #[test]
    fn parse_kw_error() {
        let source = r#"
            systems gui model;
            type color = type | blue;
            msg model istrue bool;
            connect => istrue => disconnect;
        "#;
        let parsed = parser::parse_Root(source);
        assert!(parsed.is_err());
    }

    fn validate_helper(source: &str) -> Result<(), (String, ast::Span)> {
        let root = parser::parse_Root(source).unwrap();
        ast::validate(&root)
    }

    #[test]
    fn validate_connect_loop_err() {
        let source = r#"
            systems gui model;
            msg model name bool;
            connect => name => disconnect;
            connect => connect;
        "#;
        validate_helper(source).unwrap_err();
    }

    #[test]
    fn validate_connect_disconnect_err() {
        let source = r#"
            systems gui model;
            msg model name bool;
            connect => name => disconnect;
            connect => disconnect;
        "#;
        validate_helper(source).unwrap_err();
    }

    #[test]
    fn validate_missing_connect_err() {
        let source = r#"
            systems gui model;
            msg model hi bool;
            msg gui sup bool;
            hi => sup => disconnect;
        "#;
        validate_helper(source).unwrap_err();
    }

    #[test]
    fn validate_connect_incoming_edge_err() {
        let source = r#"
            systems gui model;
            msg model hi bool;
            msg gui sup bool;
            connect => hi => disconnect;
            hi => sup => connect;
        "#;
        validate_helper(source).unwrap_err();
    }
}
