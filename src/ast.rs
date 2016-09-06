use petgraph;
use std::collections::HashSet;
use std::iter::FromIterator;

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

enum TyArg {
    Generic,
    Integer
}

fn basis() -> HashMap<String, Vec<TyArg>> {
    let mut map = HashMap::new();
    map.insert("bool".to_string(), vec![]);
    map.insert("integer".to_string(),vec![TyArg::Integer, TyArg::Integer]);
    map.insert("float".to_string(), vec![]);
    map.insert("double".to_string(), vec![]);
    map.insert("blob".to_string(), vec![]);
    map.insert("unicode".to_string(), vec![]);
    map.insert("array".to_string(), vec![TyArg::Generic, TyArg::Integer]);
    map.insert("vector".to_string(), vec![TyArg::Generic]);
    map
}

/// Walk through the AST and make sure the types and aliases are legal
pub fn validate(root: &Root) -> Result<(), (String, (usize, usize))> {
    let mut known_tycons = basis();

    fn validate_ty(ty: &Ty) -> Result<(), (String, (usize, usize))> {
        match *ty {
            Ty::IntLiteral(ref s) => {
                return Err
                panic!("AYYYY LMAO");
            },
            Ty::TyApply(ref sident, ref args) => {
                if known_tycons.contains_key(sident.node) {
                    // We're OK!
                } else {
                    return Err(
                }

                for t in args.iter() {
                    validate_ty(t);
                }
            },
            Ty::Product(ref tys) => {
                for t in tys.iter() {
                    validate_ty(t);
                }
            },
        }

        Ok(())
    }

    for tydecl in root.types.iter() {
        match *tydecl {
            TyDecl::Type(ref spanned, Sum(ref binds)) => {
                // This is always a 0 argument tycon

                let mut set = HashSet::new();
                for b in binds.iter() {
                    if set.contains(&b.0.node.as_str()) {
                        return Err(("Used the same variant name twice in a sum type".to_string(), b.0.span));
                    } else {
                        set.insert(b.0.node.as_str());
                    }
                }

            }
            TyDecl::Alias(ref spanned, ref ty) => {
            }
        }
    }

    panic!("ayy");
}
