use petgraph;

#[derive(Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: (usize, usize)
}

pub struct Root {
    pub systems: SystemsDecl,
    pub types: Vec<TyDecl>,
    pub messages: Vec<MessageDecl>,
    pub graph: Graph
}

pub struct SystemsDecl(pub Spanned<String>, pub Spanned<String>);

pub enum TyDecl {
    Type(Spanned<String>, Sum),
    Alias(Spanned<String>, Ty),
}

pub struct Sum(pub Vec<SumBind>);

pub struct SumBind(pub Spanned<String>, pub Option<Ty>);

pub enum Ty {
    IntLiteral(String),
    TyApply(Spanned<String>, Vec<Ty>),
    Product(Vec<Ty>),
}

pub struct MessageDecl {
    pub sender: Spanned<String>,
    pub name: Spanned<String>,
    pub t: Spanned<String>,
}

pub type Graph = petgraph::graph::Graph<Spanned<GraphIdent>, ()>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GraphIdent {
    Identifier(String),
    Connect,
    Disconnect,
}
