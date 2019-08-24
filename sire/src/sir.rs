use std::cmp::Ordering;
use std::fmt;

pub use rustc::hir::def_id::DefId;
pub use rustc::mir::BinOp;

pub use self::display::*;
pub use self::ty::*;

mod display;
mod ty;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuncDef {
    pub def_id: DefId,
    pub body: Expr,
    pub ty: Ty,
}

impl FuncDef {
    pub fn is_recursive(&self) -> bool {
        self.body.contains(&Expr::Value(Value::Function(self.def_id, self.ty.clone())))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Param(pub usize, pub Ty);

impl Ord for Param {
    fn cmp(&self, other: &Self) -> Ordering {
        let Param(a, _) = self;
        let Param(b, _) = other;
        a.cmp(b)
    }
}

impl PartialOrd for Param {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Value(Value),
    Apply(Box<Expr>, Vec<Expr>),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    Switch(Box<Expr>, Vec<Expr>, Vec<Expr>),
    Tuple(Vec<Expr>),
    Projection(Box<Expr>, usize),
    Uninitialized,
}

impl Expr {
    pub fn contains(&self, target: &Self) -> bool {
        *self == *target
            || match self {
                Expr::Apply(e1, e2) => e1.contains(target) || e2.iter().any(|e| e.contains(target)),
                Expr::Switch(e1, e2, e3) => {
                    e1.contains(target)
                        || e2.iter().any(|e| e.contains(target))
                        || e3.iter().any(|e| e.contains(target))
                }
                Expr::BinaryOp(_, e1, e2) => e1.contains(target) || e2.contains(target),
                Expr::Tuple(e1) => e1.iter().any(|e| e.contains(target)),
                _ => false,
            }
    }

    pub fn replace(&mut self, target: &Self, substitution: &Self) {
        if *self == *target {
            *self = substitution.clone();
        } else {
            match self {
                Expr::Apply(e1, e2) => {
                    e1.replace(target, substitution);
                    for e in e2 {
                        e.replace(target, substitution);
                    }
                }
                Expr::Switch(e1, e2, e3) => {
                    e1.replace(target, substitution);
                    for e in e2 {
                        e.replace(target, substitution);
                    }
                    for e in e3 {
                        e.replace(target, substitution);
                    }
                }
                Expr::BinaryOp(_, e1, e2) => {
                    e1.replace(target, substitution);
                    e2.replace(target, substitution);
                }
                Expr::Tuple(e1) => {
                    for e in e1 {
                        e.replace(target, substitution);
                    }
                }
                _ => (),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Arg(usize, Ty),
    Const(u128, Ty),
    Function(DefId, Ty),
    ConstParam(Param),
}
