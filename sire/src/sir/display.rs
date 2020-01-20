use super::*;

impl fmt::Display for FuncDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = match &self.ty {
            Ty::Func(_, params) => {
                params.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" ")
            }
            _ => unreachable!(),
        };

        write!(f, "(defun {:?}[{}] {} {})", self.def_id, params, self.ty, self.body)
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int(n) => write!(f, "(int {})", n),
            Ty::Uint(n) => write!(f, "(uint {})", n),
            Ty::Bool => write!(f, "bool"),
            Ty::Func(args_ty, _) => {
                write!(f, "{}", args_ty.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "),)
            }
            Ty::Tuple(fields_ty) => write!(
                f,
                "({})",
                fields_ty.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "),
            ),
        }
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Param(index, ty) = self;
        write!(f, "(p{} {})", index, ty)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Value(value) => write!(f, "{}", value),
            Expr::Apply(func, args) => write!(
                f,
                "({} {})",
                func,
                args.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
            ),
            Expr::BinaryOp(op, e1, e2) => {
                let op_string = match op {
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
                    _ => unreachable!(),
                };
                write!(f, "({} {} {})", op_string, e1, e2)
            }
            Expr::Switch(value, branches, targets) => write!(
                f,
                "(switch {} {} (else -> {}))",
                value,
                branches
                    .iter()
                    .zip(targets.iter())
                    .map(|(b, t)| format!("({} -> {})", b, t))
                    .collect::<Vec<_>>()
                    .join(" "),
                targets.last().unwrap()
            ),
            Expr::Tuple(fields) => write!(
                f,
                "(tuple {})",
                fields.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "),
            ),
            Expr::Projection(e1, i) => write!(f, "(proj {} {})", e1, i),
            Expr::Assert(e1, e2) => write!(f, "(assert {} {})", e1, e2),
            Expr::Uninitialized => write!(f, "uninitialized"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Arg(n, _) => write!(f, "_{}", n),
            Value::Const(value, ty) => write!(f, "(const {} {})", ty, value),
            Value::Function(def_id, _) => write!(f, "{:?}", def_id),
            Value::ConstParam(Param(index, _)) => write!(f, "p{}", index),
        }
    }
}
