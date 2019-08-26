use super::*;

impl Expr {
    pub fn optimize(&mut self) {
        match self {
            Expr::Value(_) => (),
            Expr::Apply(e1, e2) => {
                e1.optimize();
                for e in e2 {
                    e.optimize();
                }
            }
            Expr::BinaryOp(_, e1, e2) => {
                e1.optimize();
                e2.optimize();
            }
            Expr::Switch(e1, e2, e3) => {
                e1.optimize();
                for e in e3 {
                    e.optimize();
                }
                for e in e2 {
                    e.optimize();
                }
            }
            Expr::Tuple(e1) => {
                for e in e1 {
                    e.optimize();
                }
            }
            Expr::Projection(e1, index) => {
                if let box Expr::Tuple(fields) = e1 {
                    if let Some(field) = fields.get_mut(*index) {
                        field.optimize();
                        *self = field.clone();
                    }
                }
            }
            Expr::Just(e1) => {
                e1.optimize();
            }
            Expr::Nothing(_) => (),
            Expr::Uninitialized => (),
        }
    }
}
