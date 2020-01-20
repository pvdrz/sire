use super::*;

pub trait VisitorMut {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        self.super_mut_expr(expr)
    }

    fn visit_mut_value(&mut self, value: &mut Value) {
        self.super_mut_value(value)
    }

    fn visit_mut_apply(&mut self, func: &mut Expr, args: &mut [Expr]) {
        self.super_mut_apply(func, args)
    }

    fn visit_mut_binary_op(&mut self, bin_op: &mut BinOp, e1: &mut Expr, e2: &mut Expr) {
        self.super_mut_binary_op(bin_op, e1, e2)
    }

    fn visit_mut_switch(&mut self, expr: &mut Expr, values: &mut [Expr], results: &mut [Expr]) {
        self.super_mut_switch(expr, values, results)
    }

    fn visit_mut_tuple(&mut self, fields: &mut [Expr]) {
        self.super_mut_tuple(fields)
    }

    fn visit_mut_projection(&mut self, tuple: &mut Expr, index: usize) {
        self.super_mut_projection(tuple, index)
    }

    fn visit_mut_assert(&mut self, condition: &mut Expr, result: &mut Expr) {
        self.super_mut_assert(condition, result)
    }

    fn super_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Value(e) => self.visit_mut_value(e),
            Expr::Apply(e1, e2) => self.visit_mut_apply(e1, e2),
            Expr::BinaryOp(op, e1, e2) => self.visit_mut_binary_op(op, e1, e2),
            Expr::Switch(e1, e2, e3) => self.visit_mut_switch(e1, e2, e3),
            Expr::Tuple(e1) => self.visit_mut_tuple(e1),
            Expr::Projection(e1, index) => self.visit_mut_projection(e1, *index),
            Expr::Assert(e1, e2) => self.visit_mut_assert(e1, e2),
            Expr::Uninitialized => (),
        }
    }
    fn super_mut_value(&mut self, _: &mut Value) {
        ()
    }

    fn super_mut_apply(&mut self, func: &mut Expr, args: &mut [Expr]) {
        self.visit_mut_expr(func);
        for arg in args {
            self.visit_mut_expr(arg);
        }
    }

    fn super_mut_binary_op(&mut self, _: &mut BinOp, e1: &mut Expr, e2: &mut Expr) {
        self.visit_mut_expr(e1);
        self.visit_mut_expr(e2);
    }

    fn super_mut_switch(&mut self, expr: &mut Expr, values: &mut [Expr], results: &mut [Expr]) {
        self.visit_mut_expr(expr);
        for value in values {
            self.visit_mut_expr(value);
        }
        for result in results {
            self.visit_mut_expr(result);
        }
    }

    fn super_mut_tuple(&mut self, fields: &mut [Expr]) {
        for field in fields {
            self.visit_mut_expr(field);
        }
    }

    fn super_mut_projection(&mut self, tuple: &mut Expr, _: usize) {
        self.visit_mut_expr(tuple)
    }

    fn super_mut_assert(&mut self, condition: &mut Expr, result: &mut Expr) {
        self.visit_mut_expr(condition);
        self.visit_mut_expr(result);
    }
}
