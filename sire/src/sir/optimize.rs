use super::*;
use visitor::Visitor;

impl Expr {
    pub fn optimize(&mut self) {
        Optimizer::optimize(self);
    }
}

#[derive(Default)]
struct Optimizer {
    expr: Option<Expr>,
}

impl Optimizer {
    fn optimize(expr: &mut Expr) {
        Self::default().visit_expr(expr);
    }
}

impl Visitor for Optimizer {
    fn visit_expr(&mut self, expr: &mut Expr) {
        self.super_expr(expr);

        if let Some(new_expr) = self.expr.take() {
            *expr = new_expr;
        }
    }

    fn visit_projection(&mut self, tuple: &mut Expr, index: usize) {
        self.visit_expr(tuple);

        if let Expr::Tuple(fields) = tuple {
            if let Some(field) = fields.get(index) {
                self.expr = Some(field.clone());
            }
        } else {
            unreachable!()
        }
    }
}
