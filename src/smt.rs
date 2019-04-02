use crate::interpreter::Expr;
use rustc::mir::BinOp;

pub trait ToSmt {
    fn to_smt(&self) -> String;
}
impl ToSmt for Expr {
    fn to_smt(&self) -> String {
        match self {
            Expr::Function(fun) => fun.clone(),
            Expr::Int(value) => value.to_string(),
            Expr::Nat(value) => value.to_string(),
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
                bs[0].to_smt(),
                bs[1].to_smt()
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
            BinOp::Lt => ">",
            BinOp::Le => ">=",
            BinOp::Ne => "!=",
            BinOp::Ge => ">=",
            BinOp::Gt => ">",
            _ => unimplemented!(),
        }
        .to_owned()
    }
}
