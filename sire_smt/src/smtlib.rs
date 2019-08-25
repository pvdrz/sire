use sire::sir::*;

pub trait ToSmtlib {
    fn to_smtlib(&self) -> String;
}

impl ToSmtlib for FuncDef {
    fn to_smtlib(&self) -> String {
        let body = self.body.to_smtlib();
        let (args, params) = match &self.ty {
            Ty::Func(args, params) => (args, params),
            _ => unreachable!(),
        };

        let ret_ty = args[0].to_smtlib();

        let args_with_ty = args
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, ty)| format!("(x{} {})", i, ty.to_smtlib()))
            .chain(params.iter().map(|Param(index, ty)| format!("(p{} {})", index, ty.to_smtlib())))
            .collect::<Vec<String>>()
            .join(" ");

        let def = if self.is_recursive() { "define-fun-rec" } else { "define-fun" };
        format!(
            "({def} {name} ({args_with_ty}) {ret_ty} {body})",
            def = def,
            name = self.def_id.to_smtlib(),
            ret_ty = ret_ty,
            args_with_ty = args_with_ty,
            body = body
        )
    }
}

impl ToSmtlib for Param {
    fn to_smtlib(&self) -> String {
        let Param(index, _) = self;
        format!("p{}", index)
    }
}

impl ToSmtlib for Ty {
    fn to_smtlib(&self) -> String {
        match self {
            Ty::Bool => "Bool".to_owned(),
            Ty::Tuple(fields) => {
                let mut fields = fields.iter().rev();
                if let Some(first) = fields.next() {
                    let mut buffer = first.to_smtlib();
                    for field in fields {
                        buffer = format!("(Tuple {} {})", field, buffer);
                    }
                    buffer
                } else {
                    "Unit".to_owned()
                }
            }
            Ty::Maybe(ty) => format!("(Maybe {})", ty.to_smtlib()),
            _ => format!("(_ BitVec {})", self.bits().unwrap()),
        }
    }
}

impl ToSmtlib for DefId {
    fn to_smtlib(&self) -> String {
        format!("func_{}_{}", self.krate.as_u32(), self.index.as_u32())
    }
}

impl ToSmtlib for Value {
    fn to_smtlib(&self) -> String {
        match self {
            Value::Arg(n, _) => format!("x{}", n),
            Value::Const(b, ty) => match ty {
                Ty::Bool => format!("{}", *b != 0),
                ty => format!("(_ bv{} {})", b, ty.bits().unwrap()),
            },
            Value::Function(d, _) => d.to_smtlib(),
            Value::ConstParam(p) => p.to_smtlib(),
        }
    }
}

impl ToSmtlib for Expr {
    fn to_smtlib(&self) -> String {
        match self {
            Expr::Value(value) => value.to_smtlib(),
            Expr::BinaryOp(op, e1, e2) => {
                let smt_op = match e1.ty() {
                    Ty::Bool => match op {
                        BinOp::Eq => "=",
                        BinOp::Ne => "!=",
                        _ => unreachable!(),
                    },
                    Ty::Int(_) => match op {
                        BinOp::Add => "bvadd",
                        BinOp::Sub => "bvsub",
                        BinOp::Mul => "bvmul",
                        BinOp::Div => "bvsdiv",
                        BinOp::Rem => "bvsrem",
                        BinOp::Eq => "=",
                        BinOp::Lt => "bvslt",
                        BinOp::Le => "bvsle",
                        BinOp::Ne => "!=",
                        BinOp::Ge => "bvsge",
                        BinOp::Gt => "bvsgt",
                        _ => unreachable!(),
                    },
                    Ty::Uint(_) => match op {
                        BinOp::Add => "bvadd",
                        BinOp::Sub => "bvsub",
                        BinOp::Mul => "bvmul",
                        BinOp::Div => "bvudiv",
                        BinOp::Rem => "bvurem",
                        BinOp::Eq => "=",
                        BinOp::Lt => "bvult",
                        BinOp::Le => "bvule",
                        BinOp::Ne => "!=",
                        BinOp::Ge => "bvuge",
                        BinOp::Gt => "bvugt",
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                format!("({} {} {})", smt_op, e1.to_smtlib(), e2.to_smtlib())
            }
            Expr::Apply(f, es) => format!(
                "({} {})",
                f.to_smtlib(),
                es.iter().map(ToSmtlib::to_smtlib).collect::<Vec<_>>().join(" ")
            ),
            Expr::Switch(val, cs, bs) => {
                if let Ty::Bool = val.ty() {
                    format!("(ite {} {} {})", val.to_smtlib(), bs[1].to_smtlib(), bs[0].to_smtlib())
                } else {
                    let mut cond = bs.last().unwrap().to_smtlib();
                    for i in (0..cs.len()).rev() {
                        cond = format!(
                            "(ite (= {} {}) {} {})",
                            val.to_smtlib(),
                            cs[i].to_smtlib(),
                            bs[i].to_smtlib(),
                            cond
                        );
                    }
                    cond
                }
            }
            Expr::Tuple(fields) => {
                let mut fields = fields.iter().rev();
                if let Some(first) = fields.next() {
                    let mut buffer = first.to_smtlib();
                    for field in fields {
                        buffer = format!("(tuple {} {})", field.to_smtlib(), buffer);
                    }
                    buffer
                } else {
                    "unit".to_owned()
                }
            }
            Expr::Projection(box tuple, index) => {
                let mut buffer = tuple.to_smtlib();
                match index {
                    0 => buffer = format!("(first {})", buffer),
                    1 => buffer = format!("(second {})", buffer),
                    // FIXME: Support larger tuples
                    _ => unimplemented!(),
                }
                buffer
            }
            Expr::Just(e1) => format!("(just {})", e1.to_smtlib()),
            Expr::Nothing(_) => "nothing".to_owned(),
            _ => unimplemented!(),
        }
    }
}
