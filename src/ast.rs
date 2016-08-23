use petgraph;

pub struct TyDecl(pub String, pub TyCon);

pub enum TyCon {
    Direct(Ty),
    Sum(Vec<(String, Ty)>)
}

pub enum Ty {
    IntLiteral(String),
    TyApply(String, Vec<Ty>),
    Product(Vec<Ty>),
}

pub struct TyApply(TyCon, Vec<Ty>);

pub struct SystemsDecl(pub String, pub String);

pub struct MessageDecl {
    pub sender: String,
    pub name: String,
    pub t: String,
}

pub type Graph = petgraph::graph::Graph<GraphIdent, ()>;

#[derive(Debug, PartialEq)]
pub enum GraphIdent {
    Identifier(String),
    Connect,
    Disconnect,
}
