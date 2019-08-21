use rustc::hir::def_id::{DefIndex, CrateNum};

use sire_smt::check_equality;
use sire::sir::*;

#[test]
fn test_equality_sat() -> Result<(), Box<dyn std::error::Error>> {
    let a = FuncDef {
        def_id: DefId { krate: CrateNum::new(0), index: DefIndex::from_usize(1)},
        body: Expr::BinaryOp(
            BinOp::Add,
            Box::new(Expr::Value(Value::Arg(1, Ty::Uint(32)))),
            Box::new(Expr::Value(Value::Arg(1, Ty::Uint(32)))),
        ),
        ty: Ty::Func(vec![Ty::Uint(32), Ty::Uint(32)]),
    };

    let b = FuncDef {
        def_id: DefId { krate: CrateNum::new(0), index: DefIndex::from_usize(2)},
        body: Expr::BinaryOp(
            BinOp::Mul,
            Box::new(Expr::Value(Value::Const(2, Ty::Uint(32)))),
            Box::new(Expr::Value(Value::Arg(1, Ty::Uint(32)))),
        ),
        ty: Ty::Func(vec![Ty::Uint(32), Ty::Uint(32)]),
    };

    assert_eq!(sire_smt::CheckResult::Sat, check_equality(&a, &b)?);

    Ok(())
}

#[test]
fn test_equality_unsat()  -> Result<(), Box<dyn std::error::Error>> {
    let a = FuncDef {
        def_id: DefId { krate: CrateNum::new(0), index: DefIndex::from_usize(1)},
        body: Expr::BinaryOp(
            BinOp::Add,
            Box::new(Expr::Value(Value::Arg(1, Ty::Uint(32)))),
            Box::new(Expr::Value(Value::Arg(1, Ty::Uint(32)))),
        ),
        ty: Ty::Func(vec![Ty::Uint(32), Ty::Uint(32)]),
    };

    let b = FuncDef {
        def_id: DefId { krate: CrateNum::new(0), index: DefIndex::from_usize(2)},
        body: Expr::Value(Value::Arg(1, Ty::Uint(32))),
        ty: Ty::Func(vec![Ty::Uint(32), Ty::Uint(32)]),
    };

    assert_eq!(sire_smt::CheckResult::Unsat, check_equality(&a, &b)?);

    Ok(())
}
