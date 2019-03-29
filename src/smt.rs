use crate::mir::BinOp;
use crate::Expr;

pub trait ToSMT {
    fn to_smt(&self) -> String;
}
impl ToSMT for Expr {
    fn to_smt(&self) -> String {
        match self {
            Expr::Function(fun) => fun.clone(),
            Expr::Value(value) => value.to_string(),
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

impl ToSMT for BinOp {
    fn to_smt(&self) -> String {
        match self {
            BinOp::Eq => "=",
            BinOp::Gt => ">",
            BinOp::Lt => ">",
            BinOp::Add => "+",
            BinOp::Sub => "-",
        }
        .to_owned()
    }
}
