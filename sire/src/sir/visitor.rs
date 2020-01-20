use super::*;

pub trait Visitor {
    fn visit_expr(&mut self, expr: &Expr) {
        self.super_expr(expr)
    }

    fn visit_value(&mut self, value: &Value) {
        self.super_value(value)
    }

    fn visit_apply(&mut self, func: &Expr, args: &[Expr]) {
        self.super_apply(func, args)
    }

    fn visit_binary_op(&mut self, bin_op: &BinOp, e1: &Expr, e2: &Expr) {
        self.super_binary_op(bin_op, e1, e2)
    }

    fn visit_switch(&mut self, expr: &Expr, values: &[Expr], results: &[Expr]) {
        self.super_switch(expr, values, results)
    }

    fn visit_tuple(&mut self, fields: &[Expr]) {
        self.super_tuple(fields)
    }

    fn visit_projection(&mut self, tuple: &Expr, index: usize) {
        self.super_projection(tuple, index)
    }

    fn visit_assert(&mut self, condition: &Expr, result: &Expr) {
        self.super_assert(condition, result)
    }

    fn super_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Value(e) => self.visit_value(e),
            Expr::Apply(e1, e2) => self.visit_apply(e1, e2),
            Expr::BinaryOp(op, e1, e2) => self.visit_binary_op(op, e1, e2),
            Expr::Switch(e1, e2, e3) => self.visit_switch(e1, e2, e3),
            Expr::Tuple(e1) => self.visit_tuple(e1),
            Expr::Projection(e1, index) => self.visit_projection(e1, *index),
            Expr::Assert(e1, e2) => self.visit_assert(e1, e2),
            Expr::Uninitialized => (),
        }
    }

    fn super_value(&mut self, _: &Value) {
        ()
    }

    fn super_apply(&mut self, func: &Expr, args: &[Expr]) {
        self.visit_expr(func);
        for arg in args {
            self.visit_expr(arg);
        }
    }

    fn super_binary_op(&mut self, _: &BinOp, e1: &Expr, e2: &Expr) {
        self.visit_expr(e1);
        self.visit_expr(e2);
    }

    fn super_switch(&mut self, expr: &Expr, values: &[Expr], results: &[Expr]) {
        self.visit_expr(expr);
        for value in values {
            self.visit_expr(value);
        }
        for result in results {
            self.visit_expr(result);
        }
    }

    fn super_tuple(&mut self, fields: &[Expr]) {
        for field in fields {
            self.visit_expr(field);
        }
    }

    fn super_projection(&mut self, tuple: &Expr, _: usize) {
        self.visit_expr(tuple)
    }

    fn super_assert(&mut self, condition: &Expr, result: &Expr) {
        self.visit_expr(condition);
        self.visit_expr(result);
    }
}
