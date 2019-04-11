use crate::lang::*;

use std::collections::HashMap;

use rustc::hir::def_id::DefId;
use rustc::hir::def_id::LOCAL_CRATE;
use rustc::hir::ItemKind;
use rustc::mir::interpret::ConstValue;
use rustc::mir::*;
use rustc::ty::layout::Size;
use rustc::ty::{TyCtxt, TyKind};

pub type EvalResult<T = ()> = Result<T, EvalError>;

#[derive(Debug)]
pub struct EvalError(String);

macro_rules! eval_err {
    ($($arg:tt)*) => (EvalError(format!($($arg)*)))
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
    pub fn from_tcx<'a, 'gcx>(tcx: TyCtxt<'a, 'gcx, 'tcx>) -> EvalResult<Self> {
        let hir = tcx.hir();
        let mut mirs = HashMap::new();
        let mut funcs = HashMap::new();
        let mut def_ids = Vec::new();

        let (entry_def_id, _) = tcx.entry_fn(LOCAL_CRATE).expect("no main function found!");

        for (hir_id, item) in &hir.krate().items {
            if let ItemKind::Fn(_, _, _, _) = item.node {
                let def_id = hir.local_def_id_from_hir_id(*hir_id);
                if def_id != entry_def_id {
                    let name = tcx.def_path(def_id).to_filename_friendly_no_crate();
                    let mir = tcx.optimized_mir(def_id);
                    let mut args_ty = Vec::new();
                    for local_decl in mir.local_decls.iter().take(mir.arg_count + 1) {
                        args_ty.push(transl_tykind(&local_decl.ty.sty)?);
                    }

                    mirs.insert(def_id, mir);
                    funcs.insert(def_id, Value::Function(name, Ty::Func(args_ty)));
                    def_ids.push(def_id);
                }
            }
        }

        Ok(Interpreter {
            block: None,
            statement: 0,
            memory: HashMap::new(),
            funcs,
            mirs,
            def_id: None,
        })
    }

    pub fn eval_all(&mut self) -> EvalResult<Vec<FuncDef>> {
        let mut func_defs = Vec::new();
        let def_ids = self.mirs.keys().cloned().collect::<Vec<DefId>>();
        for def_id in def_ids {
            func_defs.push(self.eval_mir(def_id)?);
        }
        Ok(func_defs)
    }

    pub fn eval_mir(&mut self, def_id: DefId) -> EvalResult<FuncDef> {
        let (name, args_ty) = match self
            .funcs
            .get(&def_id)
            .ok_or_else(|| eval_err!("Mir wit DefId {:?} not found", def_id))?
        {
            Value::Function(name, Ty::Func(args_ty)) => (name.clone(), args_ty.clone()),
            _ => unreachable!(),
        };

        self.memory.insert(
            Place::Base(PlaceBase::Local(Local::from_usize(0))),
            Expr::Nil,
        );

        for (i, arg_ty) in args_ty.iter().enumerate().skip(1) {
            self.memory.insert(
                Place::Base(PlaceBase::Local(Local::from_usize(i))),
                Expr::Value(Value::Arg(i, arg_ty.clone())),
            );
        }

        self.def_id = Some(def_id);
        self.block = Some(BasicBlock::from_u32(0));

        self.run()?;

        for i in 1usize..args_ty.len() {
            let place = Place::Base(PlaceBase::Local(Local::from_usize(i)));
            self.memory
                .remove(&place)
                .ok_or_else(|| eval_err!("Double free error on place {:?}", place))?;
        }

        let body = self
            .memory
            .remove(&Place::Base(PlaceBase::Local(Local::from_u32(0))))
            .ok_or_else(|| eval_err!("Double free error on return place"))?;

        Ok(FuncDef {
            body,
            name: name.clone(),
            ty: Ty::Func(args_ty.clone()),
        })
    }

    fn run(&mut self) -> EvalResult {
        while self.step()? {}
        Ok(())
    }

    fn step(&mut self) -> EvalResult<bool> {
        let block_data = self
            .mirs
            .get(&self.def_id.expect("Bug: DefId should be some"))
            .expect("Bug: Mir should exist")
            .basic_blocks()
            .get(self.block.expect("Bug: Block should be some"))
            .ok_or_else(|| eval_err!("Basic block not found"))?;

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
                self.memory
                    .remove(&Place::Base(PlaceBase::Local(local)))
                    .ok_or_else(|| eval_err!("Double free error on local {:?}", local))?;
            }
            ref sk => {
                return Err(eval_err!("StatementKind {:?} is unsupported", sk));
            }
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
                    *self
                        .memory
                        .get_mut(place)
                        .ok_or_else(|| eval_err!("Place {:?} is uninitialized", place))? =
                        Expr::Apply(Box::new(func_expr), args_expr);
                    self.block = Some(*block);
                    self.statement = 0;
                    Ok(true)
                }
                None => Err(eval_err!("Call terminator does not assign")),
            },
            TerminatorKind::SwitchInt {
                ref discr,
                ref switch_ty,
                ref values,
                ref targets,
                ..
            } => {
                let discr_expr = self.eval_operand(&discr)?;
                let mut values_expr = Vec::new();
                let mut targets_expr = Vec::new();

                for i in 0..values.len() {
                    let bytes = values[i];
                    let block = targets[i];

                    let mut interpreter = self.clone();
                    interpreter.block = Some(block);
                    interpreter.statement = 0;
                    interpreter.run()?;

                    let value_expr =
                        Expr::Value(Value::Const(bytes, transl_tykind(&switch_ty.sty)?));

                    let mut target_expr = interpreter
                        .memory
                        .get(&Place::Base(PlaceBase::Local(Local::from_u32(0))))
                        .ok_or_else(|| eval_err!("Return place is uninitialized"))?
                        .clone();

                    target_expr.replace(&discr_expr, &value_expr);

                    values_expr.push(value_expr);
                    targets_expr.push(target_expr);
                }

                self.block = Some(*targets.last().unwrap());
                self.statement = 0;
                self.run()?;

                targets_expr.push(
                    self.memory
                        .get(&Place::Base(PlaceBase::Local(Local::from_u32(0))))
                        .ok_or_else(|| eval_err!("Return place is uninitialized"))?
                        .clone(),
                );

                *self
                    .memory
                    .get_mut(&Place::Base(PlaceBase::Local(Local::from_u32(0))))
                    .ok_or_else(|| eval_err!("Return place is uninitialized"))? =
                    Expr::Switch(Box::new(discr_expr), values_expr, targets_expr);

                self.block = None;
                self.statement = 0;
                Ok(false)
            }
            ref tk => Err(eval_err!("TerminatorKind {:?} is not supported", tk)),
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
                .ok_or_else(|| eval_err!("Place {:?} in reference is uninitialized", place))?
                .clone(),
            Rvalue::Use(op) => self.eval_operand(op)?,
            ref rv => return Err(eval_err!("Rvalue {:?} unsupported", rv)),
        };

        *self
            .memory
            .get_mut(place)
            .ok_or_else(|| eval_err!("Place {:?} in assignment is uninitialized", place))? = value;

        Ok(())
    }

    fn eval_operand(&self, operand: &Operand) -> EvalResult<Expr> {
        Ok(match operand {
            Operand::Move(place) | Operand::Copy(place) => self
                .memory
                .get(place)
                .ok_or_else(|| eval_err!("Place {:?} in move/copy is uninitialized", place))?
                .clone(),

            Operand::Constant(constant) => Expr::Value(
                transl_tykind(&constant.ty.sty)
                    .and_then(|ty| match constant.literal.val {
                        ConstValue::Scalar(scalar) => Ok(Value::Const(
                            scalar
                                .to_bits(Size::from_bits(ty.size().unwrap() as u64))
                                .unwrap(),
                            ty,
                        )),
                        _ => Err(eval_err!("Unsupported ConstValue")),
                    })
                    .or_else(|_| match constant.ty.sty {
                        TyKind::FnDef(ref def_id, _) => Ok(self
                            .funcs
                            .get(def_id)
                            .ok_or_else(|| eval_err!("Function with DefId {:?} not found", def_id))?
                            .clone()),
                        _ => Err(eval_err!("Unsupported TyKind in constant {:?}", constant)),
                    })?,
            ),
        })
    }
}

fn transl_tykind(ty_kind: &TyKind) -> EvalResult<Ty> {
    match ty_kind {
        TyKind::Bool => Ok(Ty::Bool),
        TyKind::Int(int_ty) => Ok(Ty::Int(int_ty.bit_width().unwrap_or(64))),
        TyKind::Uint(uint_ty) => Ok(Ty::Uint(uint_ty.bit_width().unwrap_or(64))),
        _ => Err(eval_err!("Unsupported TyKind")),
    }
}
