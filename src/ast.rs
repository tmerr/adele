use petgraph;

pub struct Root {
    pub systems: SystemsDecl,
    pub types: Vec<TyDecl>,
    pub messages: Vec<MessageDecl>,
    pub graph: Graph
}

#[derive(Debug, PartialEq)]
pub struct SystemsDecl(pub String, pub String);

pub enum TyDecl {
    Type(String, Sum),
    Alias(String, Ty),
}

pub struct Sum(pub Vec<SumBind>);

pub struct SumBind(pub String, pub Option<Ty>);

pub enum Ty {
    IntLiteral(String),
    TyApply(String, Vec<Ty>),
    Product(Vec<Ty>),
}

pub struct TyApply(String, Vec<Ty>);

#[derive(Debug, PartialEq)]
pub struct MessageDecl {
    pub sender: String,
    pub name: String,
    pub t: String,
}

pub type Graph = petgraph::graph::Graph<GraphIdent, ()>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GraphIdent {
    Identifier(String),
    Connect,
    Disconnect,
}
