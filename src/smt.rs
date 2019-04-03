use crate::interpreter::{Expr, Function, Ty};
use rustc::mir::BinOp;

pub trait ToSmt {
    fn to_smt(&self) -> String;
}

impl ToSmt for Ty {
    fn to_smt(&self) -> String {
        match self {
            Ty::Int => "Int",
            Ty::Uint => "Int",
            Ty::Bool => "Bool",
        }
        .to_owned()
    }
}

impl ToSmt for Function {
    fn to_smt(&self) -> String {
        let body = self.body.to_smt();
        let args = (0..self.args_ty.len())
            .map(|i| format!("x{}", i + 1))
            .collect::<Vec<_>>()
            .join(" ");
        let args_with_ty = self
            .args_ty
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("(x{} {})", i + 1, ty.to_smt()))
            .collect::<Vec<_>>()
            .join(" ");
        let args_ty = self
            .args_ty
            .iter()
            .map(|ty| ty.to_smt())
            .collect::<Vec<_>>()
            .join(" ");
        let ret_ty = self.ret_ty.to_smt();

        format!("(declare-fun {name} ({args_ty}) {ret_ty})\n(assert (forall ({args_with_ty}) (= ({name} {args}) {body})))\n", name = self.name, ret_ty = ret_ty, args = args, args_ty = args_ty, args_with_ty = args_with_ty, body = body)
    }
}

impl ToSmt for Expr {
    fn to_smt(&self) -> String {
        match self {
            Expr::Function(fun) => fun.clone(),
            Expr::Int(value) => value.to_string(),
            Expr::Uint(value) => value.to_string(),
            Expr::Bool(value) => value.to_string(),
            Expr::Place(id) => format!("x{}", id),
            Expr::BinaryOp(op, e1, e2) => {
                format!("({} {} {})", op.to_smt(), e1.to_smt(), e2.to_smt())
            }
            Expr::Apply(f, es) => format!(
                "({} {})",
                f.to_smt(),
                es.iter().map(|e| e.to_smt()).collect::<Vec<_>>().join(" ")
            ),
            Expr::Switch(val, cs, bs) if cs.len() == 1 => format!(
                "(ite {} {} {})",
                val.to_smt(),
                bs[1].to_smt(),
                bs[0].to_smt()
            ),
            _ => unimplemented!(),
        }
    }
}

impl ToSmt for BinOp {
    fn to_smt(&self) -> String {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Eq => "=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Ne => "!=",
            BinOp::Ge => ">=",
            BinOp::Gt => ">",
            _ => unimplemented!(),
        }
        .to_owned()
    }
}
