use rustc::mir::BinOp;

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuncDef {
    pub name: String,
    pub body: Expr,
    pub ty: Ty,
}

impl fmt::Display for FuncDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(defun {} {} {})", self.name, self.ty, self.body)
    }
}

impl FuncDef {
    pub fn is_recursive(&self) -> bool {
        self.body.contains(&Expr::Value(Value::Function(
            self.name.clone(),
            self.ty.clone(),
        )))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ty {
    Int(usize),
    Uint(usize),
    Bool,
    Func(Vec<Ty>),
}

impl Ty {
    pub fn size(&self) -> Option<usize> {
        match self {
            Ty::Int(n) | Ty::Uint(n) => Some(*n),
            Ty::Bool => Some(8),
            Ty::Func(_) => None,
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ty::Int(n) => write!(f, "(int {})", n),
            Ty::Uint(n) => write!(f, "(uint {})", n),
            Ty::Bool => write!(f, "bool"),
            Ty::Func(tys) => write!(
                f,
                "{}",
                tys.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Value(Value),
    Apply(Box<Expr>, Vec<Expr>),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    Switch(Box<Expr>, Vec<Expr>, Vec<Expr>),
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
                _ => (),
            }
        }
    }

    pub fn ty(&self) -> Ty {
        match self {
            Expr::Value(value) => value.ty(),
            Expr::Apply(e1, _) => match e1.ty() {
                Ty::Func(tys) => tys.first().unwrap().clone(),
                _ => unreachable!(),
            },
            Expr::BinaryOp(op, e1, _) => match op {
                BinOp::Eq | BinOp::Lt | BinOp::Le | BinOp::Ne | BinOp::Ge | BinOp::Gt => Ty::Bool,
                _ => e1.ty(),
            },
            Expr::Switch(_, _, es) => es.first().unwrap().ty().clone(),
            Expr::Uninitialized => unreachable!(),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Value(value) => write!(f, "{}", value),
            Expr::Apply(func, args) => write!(
                f,
                "({} {})",
                func,
                args.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
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
            Expr::Uninitialized => write!(f, "uninitialized"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Arg(usize, Ty),
    Const(u128, Ty),
    Function(String, Ty),
}

impl Value {
    pub fn ty(&self) -> Ty {
        match self {
            Value::Arg(_, ty) => ty,
            Value::Const(_, ty) => ty,
            Value::Function(_, ty) => ty,
        }
        .clone()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Arg(n, _) => write!(f, "_{}", n),
            Value::Const(value, ty) => write!(f, "(const {} {})", ty, value),
            Value::Function(name, _) => write!(f, "{}", name),
        }
    }
}
