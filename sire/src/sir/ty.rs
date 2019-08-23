use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ty {
    Int(usize),
    Uint(usize),
    Bool,
    Func(Vec<Ty>, Vec<Param>),
}

impl Ty {
    pub fn size(&self) -> Option<usize> {
        match self {
            Ty::Int(n) | Ty::Uint(n) => Some(*n),
            Ty::Bool => Some(8),
            Ty::Func(_, _) => None,
        }
    }
}

pub trait Typed {
    fn ty(&self) -> &Ty;
}

impl Typed for Param {
    fn ty(&self) -> &Ty {
        match self {
            Param::Const(_, ty) => ty,
        }
    }
}

impl Typed for Expr {
    fn ty(&self) -> &Ty {
        match self {
            Expr::Value(value) => value.ty(),
            Expr::Apply(e1, _) => match e1.ty() {
                Ty::Func(args_ty, _) => args_ty.first().unwrap(),
                _ => unreachable!(),
            },
            Expr::BinaryOp(op, e1, _) => match op {
                BinOp::Eq | BinOp::Lt | BinOp::Le | BinOp::Ne | BinOp::Ge | BinOp::Gt => &Ty::Bool,
                _ => e1.ty(),
            },
            Expr::Switch(_, _, es) => es.first().unwrap().ty(),
            Expr::Uninitialized => unreachable!(),
        }
    }
}

impl Typed for Value {
    fn ty(&self) -> &Ty {
        match self {
            Value::Arg(_, ty) => ty,
            Value::Const(_, ty) => ty,
            Value::Function(_, ty) => ty,
            Value::ConstParam(param) => param.ty(),
        }
    }
}
