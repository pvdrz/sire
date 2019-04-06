use std::collections::HashMap;

use rustc::hir::def_id::DefId;
use rustc::mir::interpret::ConstValue;
use rustc::mir::*;
use rustc::ty::layout::Size;
use rustc::ty::TyKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuncDef {
    pub name: String,
    pub body: Expr,
    pub ty: Ty,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Value(Value),
    Apply(Box<Expr>, Vec<Expr>),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    Switch(Box<Expr>, Vec<Expr>, Vec<Expr>),
    Nil,
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

impl Expr {
    pub fn ty(&self) -> Ty {
        match self {
            Expr::Value(value) => value.ty(),
            Expr::Apply(e1, _) => match e1.ty() {
                Ty::Func(tys) => tys.first().unwrap().clone(),
                _ => unreachable!(),
            },
            Expr::BinaryOp(_, e1, _) => e1.ty(),
            Expr::Switch(_, _, es) => es.first().unwrap().ty().clone(),
            Expr::Nil => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct Interpreter<'tcx> {
    block: Option<BasicBlock>,
    statement: usize,
    memory: HashMap<Place<'tcx>, Expr>,
    funcs: HashMap<DefId, Value>,
    mirs: HashMap<DefId, &'tcx Mir<'tcx>>,
    def_id: Option<DefId>,
}

impl<'tcx> Interpreter<'tcx> {
    pub fn new(funcs: HashMap<DefId, Value>, mirs: HashMap<DefId, &'tcx Mir<'tcx>>) -> Self {
        Interpreter {
            block: None,
            statement: 0,
            memory: HashMap::new(),
            funcs,
            mirs,
            def_id: None,
        }
    }

    fn mir(&self) -> &'tcx Mir<'tcx> {
        self.mirs.get(&self.def_id.unwrap()).unwrap()
    }

    pub fn eval_mir(&mut self, def_id: DefId) -> EvalResult<FuncDef> {
        let (name, args_ty) = match self.funcs.get(&def_id).unwrap() {
            Value::Function(name, Ty::Func(args_ty)) => (name.clone(), args_ty.clone()),
            _ => unreachable!(),
        };

        self.def_id = Some(def_id);
        let mir = self.mir();

        self.memory.insert(
            Place::Base(PlaceBase::Local(Local::from_usize(0))),
            Expr::Nil,
        );

        for i in 1usize..mir.arg_count + 1 {
            self.memory.insert(
                Place::Base(PlaceBase::Local(Local::from_usize(i))),
                Expr::Value(Value::Arg(i, args_ty[i].clone())),
            );
        }

        self.block = Some(BasicBlock::from_u32(0));

        self.run()?;

        for i in 1usize..mir.arg_count + 1 {
            self.memory
                .remove(&Place::Base(PlaceBase::Local(Local::from_usize(i))));
        }
        let body = self
            .memory
            .remove(&Place::Base(PlaceBase::Local(Local::from_u32(0))))
            .unwrap()
            .clone();

        Ok(FuncDef {
            body,
            name: name.clone(),
            ty: Ty::Func(args_ty.clone()),
        })
    }

    pub fn run(&mut self) -> EvalResult {
        while self.step()? {}
        Ok(())
    }

    fn step(&mut self) -> EvalResult<bool> {
        let block_data = self
            .mir()
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
                    .map(|&bytes| {
                        Expr::Value(match switch_ty.sty {
                            TyKind::Bool => Value::Const(bytes, Ty::Bool),
                            TyKind::Int(int_ty) => {
                                Value::Const(bytes, Ty::Int(int_ty.bit_width().unwrap_or(64)))
                            }
                            TyKind::Uint(uint_ty) => {
                                Value::Const(bytes, Ty::Uint(uint_ty.bit_width().unwrap_or(64)))
                            }
                            _ => unimplemented!(),
                        })
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

            Operand::Constant(constant) => Expr::Value(match constant.ty.sty {
                TyKind::Bool => match constant.literal.val {
                    ConstValue::Scalar(scalar) => {
                        Value::Const(scalar.to_bits(Size::from_bits(8)).unwrap(), Ty::Bool)
                    }
                    _ => unimplemented!(),
                },
                TyKind::Int(int_ty) => match constant.literal.val {
                    ConstValue::Scalar(scalar) => Value::Const(
                        scalar
                            .to_bits(Size::from_bits(int_ty.bit_width().unwrap_or(64) as u64))
                            .unwrap(),
                        Ty::Int(64),
                    ),
                    _ => unimplemented!(),
                },
                TyKind::Uint(uint_ty) => match constant.literal.val {
                    ConstValue::Scalar(scalar) => Value::Const(
                        scalar
                            .to_bits(Size::from_bits(uint_ty.bit_width().unwrap_or(64) as u64))
                            .unwrap(),
                        Ty::Uint(64),
                    ),
                    _ => unimplemented!(),
                },
                TyKind::FnDef(ref def_id, _) => self.funcs.get(def_id).unwrap().clone(),
                _ => unimplemented!(),
            }),
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
