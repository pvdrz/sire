#![feature(box_patterns)]

use sire::sir::*;

use crate::smtlib::ToSmtlib;

pub mod smtlib;
mod z3;

pub fn check_equality(a: &FuncDef, b: &FuncDef) -> Result<CheckResult, Box<dyn std::error::Error>> {
    if let (Ty::Func(a_args_ty, a_params), Ty::Func(b_args_ty, b_params)) = (&a.ty, &b.ty) {
        if a_args_ty == b_args_ty && a_params == b_params {
            let code = vec![
                "(declare-datatypes (T1 T2) ((Tuple (tuple (first T1) (second T2)))))".to_owned(),
                "(declare-datatypes (T1) ((Maybe nothing (just (from-maybe T1)))))".to_owned(),
                "(declare-datatypes () ((Unit (unit))))".to_owned(),
                // FIXME: Lookup instances of each datatype
                "(declare-const _ (Tuple (_ BitVec 64) Bool))".to_owned(),
                "(declare-const _ (Maybe (_ BitVec 64)))".to_owned(),
                a.to_smtlib(),
                b.to_smtlib(),
                gen_equality_assertion(a.def_id, b.def_id, a_args_ty, a_params),
                "(check-sat)".to_owned(),
            ]
            .join("\n");
            return z3::call(&code).map(CheckResult::from_string);
        }
    }
    Ok(CheckResult::Unsat)
}

#[derive(Debug, PartialEq, Eq)]
pub enum CheckResult {
    Sat,
    Unsat,
    Undecided,
    Unknown(String),
}

impl CheckResult {
    fn from_string(s: String) -> Self {
        if s == "sat\n" {
            CheckResult::Sat
        } else if s == "unsat\n" {
            CheckResult::Unsat
        } else if s == "unknown\n" {
            CheckResult::Undecided
        } else {
            CheckResult::Unknown(s)
        }
    }
}

pub fn gen_equality_assertion(a: DefId, b: DefId, args_ty: &[Ty], params: &[Param]) -> String {
    if args_ty.len() + params.len() > 1 {
        let (args_with_ty, args) = args_ty
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, ty)| (format!("(x{} {})", i, ty.to_smtlib()), format!("x{}", i)))
            .chain(params.iter().map(|Param(index, ty)| {
                (format!("(p{} {})", index, ty.to_smtlib()), format!("p{}", index))
            }))
            .unzip::<String, String, Vec<String>, Vec<String>>();

        let args_with_ty = args_with_ty.join(" ");
        let args = args.join(" ");

        format!(
            "(assert (forall ({}) (= ({} {}) ({} {}))))",
            args_with_ty,
            a.to_smtlib(),
            args,
            b.to_smtlib(),
            args
        )
    } else {
        format!("(assert (= {} {}))", a.to_smtlib(), b.to_smtlib(),)
    }
}
