use std::collections::HashMap;

use rustc::hir::def_id::DefId;
use rustc::mir::interpret::ConstValue;
use rustc::mir::*;
use rustc::ty::TyKind;
use syntax::ast::{IntTy, UintTy};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Place(usize),
    Int(i64),
    Nat(u64),
    Bool(bool),
    Function(String),
    Apply(Box<Expr>, Vec<Expr>),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    Switch(Box<Expr>, Vec<Expr>, Vec<Expr>),
    Nil,
}

#[derive(Clone, Debug)]
pub struct Interpreter<'tcx> {
    mir: Option<&'tcx Mir<'tcx>>,
    block: Option<BasicBlock>,
    statement: usize,
    memory: HashMap<Place<'tcx>, Expr>,
    names: HashMap<DefId, String>,
}

impl<'tcx> Interpreter<'tcx> {
    pub fn new(names: HashMap<DefId, String>) -> Self {
        Interpreter {
            mir: None,
            block: None,
            statement: 0,
            memory: HashMap::new(),
            names,
        }
    }

    pub fn eval_mir(&mut self, mir: &'tcx Mir<'tcx>) -> EvalResult<Expr> {
        self.memory.insert(
            Place::Base(PlaceBase::Local(Local::from_usize(0))),
            Expr::Nil,
        );

        for i in 1usize..mir.arg_count + 1 {
            self.memory.insert(
                Place::Base(PlaceBase::Local(Local::from_usize(i))),
                Expr::Place(i),
            );
        }

        self.mir = Some(mir);
        self.block = Some(BasicBlock::from_u32(0));
        self.run()?;
        for i in 1usize..mir.arg_count + 1 {
            self.memory
                .remove(&Place::Base(PlaceBase::Local(Local::from_usize(i))));
        }

        Ok(self
            .memory
            .remove(&Place::Base(PlaceBase::Local(Local::from_u32(0))))
            .unwrap()
            .clone())
    }

    pub fn run(&mut self) -> EvalResult {
        while self.step()? {}
        Ok(())
    }

    fn step(&mut self) -> EvalResult<bool> {
        let block_data = self
            .mir
            .expect("MIR should be some.")
            .basic_blocks()
            .get(self.block.expect("Block should be some."))
            .expect("BlockData should be some.");

        match block_data.statements.get(self.statement) {
            Some(statement) => self.eval_statement(statement),
            None => self.eval_terminator(block_data.terminator()),
        }
    }

    fn eval_statement(&mut self, statement: &Statement<'tcx>) -> EvalResult<bool> {
        match statement.kind {
            StatementKind::Assign(ref place, ref rvalue) => {
                self.eval_rvalue_into_place(rvalue, place)?;
            }
            StatementKind::StorageLive(local) => {
                self.memory
                    .insert(Place::Base(PlaceBase::Local(local)), Expr::Nil);
            }
            StatementKind::StorageDead(local) => {
                self.memory.remove(&Place::Base(PlaceBase::Local(local)));
            }
            _ => unimplemented!(),
        };
        self.statement += 1;
        Ok(true)
    }

    fn eval_terminator(&mut self, terminator: &Terminator<'tcx>) -> EvalResult<bool> {
        match terminator.kind {
            TerminatorKind::Return => {
                self.block = None;
                self.statement = 0;
                Ok(false)
            }
            TerminatorKind::Goto { target } => {
                self.block = Some(target);
                self.statement = 0;
                Ok(true)
            }
            TerminatorKind::Call {
                ref func,
                ref args,
                ref destination,
                ..
            } => match destination {
                Some((place, block)) => {
                    let func_expr = self.eval_operand(func)?;
                    let mut args_expr = Vec::new();
                    for op in args {
                        args_expr.push(self.eval_operand(op)?);
                    }
                    *self.memory.get_mut(place).expect("Memory should be some.") =
                        Expr::Apply(Box::new(func_expr), args_expr);
                    self.block = Some(*block);
                    self.statement = 0;
                    Ok(true)
                }
                None => unimplemented!(),
            },
            TerminatorKind::SwitchInt {
                ref discr,
                ref switch_ty,
                ref values,
                ref targets,
                ..
            } => {
                let discr_expr = self.eval_operand(&discr)?;
                let values_expr = values
                    .iter()
                    .map(|&bytes| match switch_ty.sty {
                        TyKind::Bool => Expr::Bool(bytes != 0),
                        TyKind::Int(IntTy::I64) => Expr::Int(bytes as i64),
                        TyKind::Uint(UintTy::U64) => Expr::Nat(bytes as u64),
                        _ => unimplemented!(),
                    })
                    .collect::<Vec<_>>();
                let targets_expr = targets
                    .iter()
                    .map(|block| {
                        let mut interpreter = self.clone();
                        interpreter.block = Some(*block);
                        interpreter.statement = 0;
                        interpreter.run().unwrap();
                        interpreter
                            .memory
                            .get(&Place::Base(PlaceBase::Local(Local::from_u32(0))))
                            .unwrap()
                            .clone()
                    })
                    .collect::<Vec<_>>();

                self.memory.clear();
                self.memory.insert(
                    Place::Base(PlaceBase::Local(Local::from_u32(0))),
                    Expr::Switch(Box::new(discr_expr), values_expr, targets_expr),
                );

                self.block = None;
                self.statement = 0;
                Ok(false)
            }
            _ => unimplemented!(),
        }
    }

    fn eval_rvalue_into_place(&mut self, rvalue: &Rvalue<'tcx>, place: &Place<'tcx>) -> EvalResult {
        let value = match rvalue {
            Rvalue::BinaryOp(bin_op, op1, op2) => Expr::BinaryOp(
                *bin_op,
                Box::new(self.eval_operand(op1)?),
                Box::new(self.eval_operand(op2)?),
            ),
            Rvalue::Ref(_, BorrowKind::Shared, place) => self
                .memory
                .get(place)
                .expect("Reference should be some.")
                .clone(),
            Rvalue::Use(op) => self.eval_operand(op)?,
            _ => unimplemented!(),
        };

        *self.memory.get_mut(place).expect("Place should be some.") = value;
        Ok(())
    }

    fn eval_operand(&self, operand: &Operand) -> EvalResult<Expr> {
        Ok(match operand {
            Operand::Move(place) | Operand::Copy(place) => self
                .memory
                .get(place)
                .expect("Place in operand should be some.")
                .clone(),

            Operand::Constant(constant) => match constant.ty.sty {
                TyKind::Bool => match constant.literal.val {
                    ConstValue::Scalar(scalar) => Expr::Bool(scalar.to_bool().unwrap()),
                    _ => unimplemented!(),
                },
                TyKind::Int(IntTy::I64) => match constant.literal.val {
                    ConstValue::Scalar(scalar) => Expr::Int(scalar.to_i64().unwrap()),
                    _ => unimplemented!(),
                },
                TyKind::Uint(UintTy::U64) => match constant.literal.val {
                    ConstValue::Scalar(scalar) => Expr::Nat(scalar.to_u64().unwrap()),
                    _ => unimplemented!(),
                },
                TyKind::FnDef(ref def_id, _) => Expr::Function(
                    self.names
                        .get(def_id)
                        .expect("DefId should be some.")
                        .clone(),
                ),
                _ => unimplemented!(),
            },
        })
    }
}

pub type EvalResult<T = ()> = Result<T, EvalError>;

#[derive(Debug)]
pub struct EvalError(String);

impl EvalError {
    pub fn new(inner: &str) -> Self {
        EvalError(inner.to_owned())
    }
}
