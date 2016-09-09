use petgraph;
use std::collections::{HashSet, HashMap};

pub type Span = (usize, usize);

#[derive(Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span
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

    /// If it has no argument it may either be a type or an alias
    TyApply(Spanned<String>, Vec<Ty>),

    Product(Vec<Ty>),
}

pub struct MessageDecl {
    pub sender: Spanned<String>,
    pub name: Spanned<String>,
    pub t: Ty,
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

fn validate_ty(ty: &Ty, known_tycons: &HashMap<String, Vec<TyArg>>, known_aliases: &HashSet<&str>) -> Result<(), (String, Span)> {
    match *ty {
        Ty::IntLiteral(ref s) => {
            // Range check?
        },
        Ty::TyApply(ref sident, ref args) => {
            if let Some(tyargs) = known_tycons.get(&sident.node) {
                if args.len() == tyargs.len() {
                    for (arg, expected_ty) in args.iter().zip(tyargs) {
                        let typesmatch = match (arg, expected_ty) {
                            (&Ty::IntLiteral(_), &TyArg::Integer) => true,
                            (&Ty::TyApply(_, _), &TyArg::Generic) => true,
                            (&Ty::Product(_), &TyArg::Generic) => true,
                            _ => false,
                        };

                        if !typesmatch {
                            return Err(("Type argument mismatch".to_string(), sident.span));
                        }

                        try!(validate_ty(arg, known_tycons, known_aliases));
                    }
                } else {
                    let text = format!("Expected {} type args but got {}",
                                       tyargs.len(), args.len()).to_string();
                    return Err((text, sident.span));
                }
            } else if !(args.len() == 0 && known_aliases.contains(sident.node.as_str())) {
                let text = format!("Referred to an unknown type {}", sident.node).to_string();
                return Err((text, sident.span));
            }
        },
        Ty::Product(ref tys) => {
            assert!(tys.len() >= 2, "Internal error: a product should always have at least 2 elements. This is a bug.");
            for t in tys.iter() {
                try!(validate_ty(t, known_tycons, known_aliases));
            }
        },
    }

    Ok(())
}

/// If this check completes without errors, the ast is considered correct.
pub fn validate<'a>(root: &'a Root) -> Result<(), (String, Span)> {
    let mut known_tycons = basis();
    let mut known_aliases: HashSet<&'a str> = HashSet::new();

    // No checking needed for root.systems.

    for tydecl in root.types.iter() {
        match *tydecl {
            TyDecl::Type(ref spanned, Sum(ref binds)) => {
                let mut set: HashSet<&'a str> = HashSet::new();
                for &SumBind(ref name, ref sumbinds) in binds.iter() {
                    if set.contains(name.node.as_str()) {
                        return Err(("Used the same variant name twice in a sum type".to_string(), name.span));
                    } else {
                        set.insert(&name.node);
                    }
                    for sumbind in sumbinds {
                        try!(validate_ty(sumbind, &known_tycons, &known_aliases));
                    }
                }
                if known_tycons.contains_key(spanned.node.as_str()) ||
                   known_aliases.contains(spanned.node.as_str()) {
                    return Err(("Tried to use the same type or alias name twice".to_string(), spanned.span));
                }
                known_tycons.insert(spanned.node.clone(), vec![]);
            }
            TyDecl::Alias(ref spanned, ref ty) => {
                try!(validate_ty(ty, &known_tycons, &known_aliases));
                if known_tycons.contains_key(spanned.node.as_str()) ||
                   known_aliases.contains(spanned.node.as_str()) {
                    return Err(("Tried to use the same type or alias name twice".to_string(), spanned.span));
                }
                known_aliases.insert(&spanned.node);
            }
        }
    }

    let mut msgname_to_sys: HashMap<&'a str, &'a str> = HashMap::new();
    for message in root.messages.iter() {
        if message.sender.node != root.systems.0.node &&
           message.sender.node != root.systems.1.node {
            return Err(("Expected message sender to be a known system".to_string(), message.sender.span))
        }

        if msgname_to_sys.contains_key(message.name.node.as_str()) {
            return Err(("Message names must be unique".to_string(), message.name.span));
        }
        msgname_to_sys.insert(&message.name.node, &message.sender.node);

        if let Ty::IntLiteral(_) = message.t {
            return Err(("Unexpected integer".to_string(), message.name.span));
        }
        try!(validate_ty(&message.t, &known_tycons, &known_aliases));
    }


    let g = &root.graph;

    if let Some(connect_idx) = g.node_indices().find(|&i| g.node_weight(i).unwrap().node == GraphIdent::Connect) {
        let connect_span = g.node_weight(connect_idx).unwrap().span;
        if g.neighbors_directed(connect_idx, petgraph::EdgeDirection::Outgoing).count() == 0 {
            return Err(("The connect node must have at least one outgoing edge".to_string(), connect_span));
        }
        for n in g.neighbors_directed(connect_idx, petgraph::EdgeDirection::Outgoing) {
            match g.node_weight(n).unwrap().node {
                GraphIdent::Identifier(_) => (),
                _ => return Err(("The connect node can't have edges to disconnect or itself".to_string(), (0, 0)))
            }
        }
        if g.neighbors_directed(connect_idx, petgraph::EdgeDirection::Incoming).count() > 0 {
            return Err(("The connect node should have no incoming edges".to_string(), connect_span));
        }
    } else {
        return Err(("The connect node must be present in the graph".to_string(), (0, 0)));
    }

    if let Some(disconnect_idx) = g.node_indices().find(|&i| g.node_weight(i).unwrap().node == GraphIdent::Disconnect) {
        let disconnect_span = g.node_weight(disconnect_idx).unwrap().span;
        if g.neighbors_directed(disconnect_idx, petgraph::EdgeDirection::Outgoing).count() != 0 {
            return Err(("The disconnect node should have no outgoing edges".to_string(), disconnect_span));
        }
    }

    for i in g.node_indices() {
        for j in g.neighbors(i) {
            match (&g.node_weight(i).unwrap().node, &g.node_weight(j).unwrap().node) {
                (&GraphIdent::Identifier(ref s1), &GraphIdent::Identifier(ref s2)) => {
                    if s1 == s2 {
                        let text = format!("For now nodes must alternate systems, but {} -> {} both use the same system.", s1, s2);
                        return Err((text.to_string(), (0, 0)));
                    }
                }
                _ => (),
            }
        }
    }

    Ok(())
}
