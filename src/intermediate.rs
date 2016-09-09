/// A transformed AST that represents the code for a particular system,
/// with skeleton functions, and identifiers that respect naming conventions.
/// This is in preparation for code generation.

use ast;
use std::collections::HashMap;

pub struct Root {
    pub types: Vec<TyDecl>,
    pub functions: Vec<FunctionSkeleton>
}

pub struct SystemsDecl(String, String);

pub enum TyDecl {
    Type(String, Sum),
    Alias(String, Ty),
}

type Sum = Vec<SumBind>;

pub struct SumBind(pub String, pub Option<Ty>);

pub enum Ty {
    IntLiteral(String),
    TyApply(String, Vec<Ty>),
    Product(Vec<Ty>),
}

pub struct FunctionSkeleton {
    name: String,

    /// The argument type of the function.
    /// Note that the special connect node takes no arguments.
    arg: Option<Ty>,

    /// The return type of the function.
    /// This is basically a TyDecl::Type.
    ret: (String, Sum),
}

#[derive(Copy, Clone)]
pub struct NamingConvention {
    types: IdentStyle,
    aliases: IdentStyle,
    variant_labels: IdentStyle,
    function_names: IdentStyle,
}

#[derive(Copy, Clone)]
pub enum IdentStyle {
    Pascal,
    Camel,
    Underscore
}

fn convert_ident(ident: &str, style: IdentStyle) -> String {
    match style {
        IdentStyle::Pascal => {
            let mut acc = String::new();
            for word in ident.split("_") {
                let mut letters = word.chars();
                let first: String = letters.next().unwrap().to_uppercase().collect();
                acc.push_str(&first);
                let rest: String = letters.collect();
                acc.push_str(&rest);
            }
            acc
        },
        IdentStyle::Camel => {
            let mut acc = String::new();
            let mut words = ident.split("_");
            acc.push_str(words.next().unwrap());
            for word in words {
                let mut letters = word.chars();
                let first: String = letters.next().unwrap().to_uppercase().collect();
                acc.push_str(&first);
                let rest: String = letters.collect();
                acc.push_str(&rest);
            }
            acc
        },
        IdentStyle::Underscore => {
            ident.to_string()
        },
    }
}

fn convert_ty(ty: &ast::Ty, naming: NamingConvention) -> Ty {
    match *ty {
        ast::Ty::IntLiteral(ref s) => {
            Ty::IntLiteral(s.clone())
        },
        ast::Ty::TyApply(ref spanned, ref tys) => {
            Ty::TyApply(convert_ident(&spanned.node, naming.types), convert_tys(tys, naming))
        },
        ast::Ty::Product(ref tys) => {
            Ty::Product(convert_tys(tys, naming))
        },
    }
}

fn convert_tys(tys: &[ast::Ty], naming: NamingConvention) -> Vec<Ty> {
    tys.iter().map(|ty| convert_ty(ty, naming)).collect()
}

fn convert_sumbind(&ast::SumBind(ref name, ref ty): &ast::SumBind, naming: NamingConvention) -> SumBind {
    let ident = convert_ident(&name.node, naming.variant_labels);
    let maybe_ty = if let &Some(ref t) = ty {
        Some(convert_ty(&t, naming))
    } else {
        None
    };
    SumBind(ident, maybe_ty)
}

fn convert_tydecls(tydecls: &[ast::TyDecl], naming: NamingConvention) -> Vec<TyDecl> {
    tydecls.iter().map(|tydecl| {
        match *tydecl {
            ast::TyDecl::Type(ref spanned, ast::Sum(ref sum)) => {
                let sum_prime = sum.iter().map(|bind| {
                    convert_sumbind(&bind, naming)
                }).collect();
                TyDecl::Type(convert_ident(&spanned.node, naming.types), sum_prime)
            },
            ast::TyDecl::Alias(ref spanned, ref ty) => {
                TyDecl::Alias(convert_ident(&spanned.node, naming.aliases),
                              convert_ty(&ty, naming))
            }
        }
    }).collect()
}

/// Return a recv_xyz function skeleton for the given node and system.
/// It returns None if the node's sender is itself, or if the node is a disconnect.
fn create_function(system: &str,
                   current_ident: &ast::GraphIdent,
                   neighbor_idents: &[&ast::GraphIdent],
                   message_map: &HashMap<&str, &ast::MessageDecl>,
                   naming: NamingConvention) -> FunctionSkeleton {
    let (basename, arg) = match *current_ident {
        ast::GraphIdent::Identifier(ref s) => {
            let msg = message_map.get(s.as_str()).unwrap();
            let ty = convert_ty(&msg.t, naming);
            (s.as_str(), Some(ty))
        },
        ast::GraphIdent::Connect => {
            ("connect", None)
        },
        ast::GraphIdent::Disconnect => unreachable!(),
    };
    let name = convert_ident(&("recv_".to_string() + basename), naming.function_names);

    let mut sumbinds: Vec<SumBind> = vec![];
    for n in neighbor_idents {
        match **n {
            ast::GraphIdent::Identifier(ref s) => {
                let ty0 = &message_map.get(s.as_str()).unwrap().t;
                let ty1 = convert_ty(&ty0, naming);
                sumbinds.push(SumBind(s.to_string(), Some(ty1)));
            },
            ast::GraphIdent::Connect => {
                unreachable!("connect can't have incoming edges")
            },
            ast::GraphIdent::Disconnect => {
                let name = convert_ident("disconnect", naming.variant_labels);
                sumbinds.push(SumBind(name, None));
            },
        }
    }
    let sumname = convert_ident(&(basename.to_string() + "_response"),
                                naming.variant_labels);
    FunctionSkeleton {
        name: name,
        arg: arg,
        ret: (sumname, sumbinds),
    }
}

fn create_functions<'a>(messages: &'a [ast::MessageDecl], g: &ast::Graph,
                        system: &str, naming: NamingConvention) -> Vec<FunctionSkeleton> {
    let mut message_map: HashMap<&'a str, &ast::MessageDecl> = HashMap::new();
    for msg in messages {
        message_map.insert(&msg.name.node, msg);
    }

    let mut result = vec![];
    for i in g.node_indices() {
        let current_ident = &g.node_weight(i).unwrap().node;
        let neighbor_idents: Vec<_> = g.neighbors(i).map(|n| &g.node_weight(n).unwrap().node).collect();
        let sender = match *current_ident {
            ast::GraphIdent::Identifier(ref s) => {
                &message_map.get(s.as_str()).unwrap().sender.node
            },
            ast::GraphIdent::Connect => {
                if let Some(&&ast::GraphIdent::Identifier(ref s)) = neighbor_idents.get(0) {
                    &message_map.get(s.as_str()).unwrap().sender.node
                } else {
                    panic!("connect must have at least one outgoing edge to an identifier");
                }
            },
            ast::GraphIdent::Disconnect => {
                continue
            },
        };
        if sender != system {
            let f = create_function(system, current_ident, &neighbor_idents[..], &message_map, naming);
            result.push(f);
        }
    }
    result
}

pub fn intermediate_ast(root: &ast::Root, system: &str, naming: NamingConvention) -> Root {
    assert!(system == root.systems.0.node || system == root.systems.1.node);

    Root {
        types: convert_tydecls(&root.types[..], naming),
        functions: create_functions(&root.messages[..], &root.graph, system, naming),
    }
}

#[test]
fn test_convert_ident_pascal_letter() {
    assert_eq!(&convert_ident("x", IdentStyle::Pascal), "X");
}

#[test]
fn test_convert_ident_pascal_word() {
    assert_eq!(&convert_ident("test", IdentStyle::Pascal), "Test");
}
    
#[test]
fn test_convert_ident_pascal_words() {
    assert_eq!(&convert_ident("this_is_a_test", IdentStyle::Pascal), "ThisIsATest");
}

#[test]
fn test_convert_ident_camel_letter() {
    assert_eq!(&convert_ident("x", IdentStyle::Camel), "x");
}

#[test]
fn test_convert_ident_camel_word() {
    assert_eq!(&convert_ident("test", IdentStyle::Camel), "test");
}

#[test]
fn test_convert_ident_camel_words() {
    assert_eq!(&convert_ident("this_is_a_test", IdentStyle::Camel), "thisIsATest");
}

#[test]
fn test_convert_ident_underscore() {
    assert_eq!(&convert_ident("this_is_a_test", IdentStyle::Underscore), "this_is_a_test");
}
