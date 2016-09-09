/// A transformed AST that represents the code for a particular system,
/// with skeleton functions, and identifiers that respect naming conventions.
/// This is ins preparation for code generation.

use ast;

pub struct Root {
    pub types: Vec<TyDecl>,
    pub functions: Vec<Function>
}

pub struct SystemsDecl(String, String);

pub enum TyDecl {
    Type(String, Vec<SumBind>),
    Alias(String, Ty),
}

pub struct SumBind(pub String, pub Option<Ty>);

pub enum Ty {
    IntLiteral(String),
    TyApply(String, Vec<Ty>),
    Product(Vec<Ty>),
}

pub struct Function {
    name: String,
    args: Vec<Ty>,
    ret: Ty,
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

fn create_functions(messages: &[ast::MessageDecl], graph: &ast::Graph,
                    system: &str, naming: NamingConvention) -> Vec<Function> {
}

pub fn intermediate_ast(root: &ast::Root, system: &str, naming: NamingConvention) -> Root {
    assert!(system == root.systems.0.node || system == root.systems.1.node);

    Root {
        types: convert_tydecls(&root.types[..], naming),
        functions: create_functions(&root.messages[..], &root.graph, system, naming),
    }
}
