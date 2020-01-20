use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ty {
    Int(usize),
    Uint(usize),
    Bool,
    Func(Vec<Ty>, Vec<Param>),
    Tuple(Vec<Ty>),
}

impl Ty {
    pub fn bits(&self) -> Option<usize> {
        match self {
            Ty::Int(n) | Ty::Uint(n) => Some(*n),
            Ty::Bool => Some(8),
            Ty::Func(_, _) => None,
            Ty::Tuple(fields_ty) => {
                let mut total = 0;
                for ty in fields_ty {
                    total += ty.bits()?;
                }
                Some(total)
            }
        }
    }

    pub fn bytes(&self) -> Option<usize> {
        self.bits().map(|x| x / 8)
    }
}

pub trait Typed {
    fn ty(&self) -> Ty;
}

impl Typed for Param {
    fn ty(&self) -> Ty {
        let Param(_, ty) = self;
        ty.clone()
    }
}

impl Typed for Expr {
    fn ty(&self) -> Ty {
        match self {
            Expr::Value(value) => value.ty(),
            Expr::Apply(e1, _) => match e1.ty() {
                Ty::Func(args_ty, _) => args_ty.first().unwrap().clone(),
                _ => unreachable!(),
            },
            Expr::BinaryOp(op, e1, _) => match op {
                BinOp::Eq | BinOp::Lt | BinOp::Le | BinOp::Ne | BinOp::Ge | BinOp::Gt => Ty::Bool,
                _ => e1.ty(),
            },
            Expr::Switch(_, _, e1) => e1.first().unwrap().ty(),
            Expr::Tuple(e1) => Ty::Tuple(e1.iter().map(|e| e.ty()).collect()),
            Expr::Projection(e1, i) => match **e1 {
                Expr::Tuple(ref fields) => fields.get(*i).unwrap().ty(),
                _ => unreachable!(),
            },
            Expr::Assert(_, e1) => e1.ty(),
            Expr::Uninitialized => unreachable!(),
        }
    }
}

impl Typed for Value {
    fn ty(&self) -> Ty {
        match self {
            Value::Arg(_, ty) => ty.clone(),
            Value::Const(_, ty) => ty.clone(),
            Value::Function(_, ty) => ty.clone(),
            Value::ConstParam(param) => param.ty(),
        }
    }
}
