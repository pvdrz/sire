use super::*;

pub trait Visitor {
    fn visit_expr(&mut self, expr: &mut Expr) {
        self.super_expr(expr)
    }

    fn visit_value(&mut self, value: &mut Value) {
        self.super_value(value)
    }

    fn visit_apply(&mut self, func: &mut Expr, args: &mut [Expr]) {
        self.super_apply(func, args)
    }

    fn visit_binary_op(&mut self, bin_op: &mut BinOp, e1: &mut Expr, e2: &mut Expr) {
        self.super_binary_op(bin_op, e1, e2)
    }

    fn visit_switch(&mut self, expr: &mut Expr, values: &mut [Expr], results: &mut [Expr]) {
        self.super_switch(expr, values, results)
    }

    fn visit_tuple(&mut self, fields: &mut [Expr]) {
        self.super_tuple(fields)
    }

    fn visit_projection(&mut self, tuple: &mut Expr, index: usize) {
        self.super_projection(tuple, index)
    }

    fn visit_just(&mut self, expr: &mut Expr) {
        self.super_just(expr)
    }

    fn super_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Value(e) => self.visit_value(e),
            Expr::Apply(e1, e2) => self.visit_apply(e1, e2),
            Expr::BinaryOp(op, e1, e2) => self.visit_binary_op(op, e1, e2),
            Expr::Switch(e1, e2, e3) => self.visit_switch(e1, e2, e3),
            Expr::Tuple(e1) => self.visit_tuple(e1),
            Expr::Projection(e1, index) => self.visit_projection(e1, *index),
            Expr::Just(e1) => self.visit_just(e1),
            Expr::Nothing(_) => (),
            Expr::Uninitialized => (),
        }
    }
    fn super_value(&mut self, _: &mut Value) {
        ()
    }

    fn super_apply(&mut self, func: &mut Expr, args: &mut [Expr]) {
        self.visit_expr(func);
        for arg in args {
            self.visit_expr(arg);
        }
    }

    fn super_binary_op(&mut self, _: &mut BinOp, e1: &mut Expr, e2: &mut Expr) {
        self.visit_expr(e1);
        self.visit_expr(e2);
    }

    fn super_switch(&mut self, expr: &mut Expr, values: &mut [Expr], results: &mut [Expr]) {
        self.visit_expr(expr);
        for value in values {
            self.visit_expr(value);
        }
        for result in results {
            self.visit_expr(result);
        }
    }

    fn super_tuple(&mut self, fields: &mut [Expr]) {
        for field in fields {
            self.visit_expr(field);
        }
    }

    fn super_projection(&mut self, tuple: &mut Expr, _: usize) {
        self.visit_expr(tuple)
    }

    fn super_just(&mut self, expr: &mut Expr) {
        self.visit_expr(expr)
    }
}
