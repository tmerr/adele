#[derive(Debug, PartialEq)]
pub enum GraphIdent {
    Connect,
    Disconnect,
    Identifier(String)
}
