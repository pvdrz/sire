use std::collections::HashMap;

use rustc::hir::def_id::DefId;
use rustc::mir::interpret::{ConstValue, InterpResult};
use rustc::mir::*;
use rustc::ty::{layout::Size, TyCtxt, TyKind};
use rustc::{err_unsup, err_unsup_format};

use crate::analysis::find_loop;
use crate::sir::*;

#[derive(Clone)]
pub struct Evaluator<'tcx> {
    block: Option<BasicBlock>,
    statement: usize,
    memory: HashMap<Place<'tcx>, Expr>,
    def_id: Option<DefId>,
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> Evaluator<'tcx> {
    pub fn from_tcx(tcx: TyCtxt<'tcx>) -> InterpResult<'tcx, Self> {
        Ok(Evaluator {
            block: None,
            statement: 0,
            memory: HashMap::new(),
            def_id: None,
            tcx,
        })
    }

    pub fn eval_mir(&mut self, def_id: DefId) -> InterpResult<'tcx, FuncDef> {
        let mir = self.tcx.optimized_mir(def_id);

        if find_loop(mir).is_some() {
            return Err(err_unsup_format!("The function {:?} contains loops", def_id).into());
        }

        let args_ty = mir
            .local_decls
            .iter()
            .take(mir.arg_count + 1)
            .map(|ld| self.transl_tykind(&ld.ty.sty))
            .collect::<InterpResult<'_, Vec<Ty>>>()?;

        self.memory
            .insert(Local::from_usize(0).into(), Expr::Uninitialized);

        for (i, arg_ty) in args_ty.iter().enumerate().skip(1) {
            self.memory.insert(
                Place {
                    base: PlaceBase::Local(Local::from_usize(i)),
                    projection: None,
                },
                Expr::Value(Value::Arg(i, arg_ty.clone())),
            );
        }

        let locals_len = mir.local_decls.len();
        let (live, dead) = self.check_storages(&def_id);

        for i in args_ty.len()..locals_len {
            let local = Local::from_usize(i);
            if !live.contains(&local) {
                self.memory
                    .insert(Local::from_usize(i).into(), Expr::Uninitialized);
            }
        }

        self.def_id = Some(def_id);
        self.block = Some(BasicBlock::from_u32(0));

        self.run()?;

        for i in 1usize..args_ty.len() {
            let place = Place {
                base: PlaceBase::Local(Local::from_usize(i)),
                projection: None,
            };
            self.memory.remove(&place).ok_or_else(|| {
                err_unsup_format!("Double free error on place {:?} argument", place)
            })?;
        }

        for i in args_ty.len()..locals_len {
            let local = Local::from_usize(i);
            if !dead.contains(&local) {
                let place: Place<'_> = local.into();
                self.memory.remove(&place).ok_or_else(|| {
                    err_unsup_format!("Double free error on place {:?} local", place)
                })?;
            }
        }

        let body = self
            .memory
            .remove(&Local::from_usize(0).into())
            .ok_or_else(|| err_unsup_format!("Double free error on return place"))?;

        if self.memory.is_empty() {
            Ok(FuncDef {
                body,
                def_id,
                ty: Ty::Func(args_ty.clone()),
            })
        } else {
            Err(err_unsup_format!("Memory is not empty after execution").into())
        }
    }

    fn run(&mut self) -> InterpResult<'tcx> {
        while self.step()? {}
        Ok(())
    }

    fn step(&mut self) -> InterpResult<'tcx, bool> {
        let block_data = self
            .tcx
            .optimized_mir(self.def_id.expect("Bug: DefId should be some"))
            .basic_blocks()
            .get(self.block.expect("Bug: Block should be some"))
            .ok_or_else(|| err_unsup_format!("Basic block not found"))?;

        match block_data.statements.get(self.statement) {
            Some(statement) => self.eval_statement(statement),
            None => self.eval_terminator(block_data.terminator()),
        }
    }

    fn eval_statement(&mut self, statement: &Statement<'tcx>) -> InterpResult<'tcx, bool> {
        match statement.kind {
            StatementKind::Assign(ref place, ref rvalue) => {
                self.eval_rvalue_into_place(rvalue, place)?;
            }
            StatementKind::StorageLive(local) => {
                self.memory.insert(local.into(), Expr::Uninitialized);
            }
            StatementKind::StorageDead(local) => {
                self.memory
                    .remove(&local.into())
                    .ok_or_else(|| err_unsup_format!("Double free error on local {:?}", local))?;
            }
            ref sk => {
                return Err(err_unsup_format!("StatementKind {:?} is unsupported", sk).into());
            }
        };
        self.statement += 1;
        Ok(true)
    }

    fn eval_terminator(&mut self, terminator: &Terminator<'tcx>) -> InterpResult<'tcx, bool> {
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
            } => {
                match destination {
                    Some((place, block)) => {
                        let func_expr = self.eval_operand(func)?;
                        let mut args_expr = Vec::new();
                        for op in args {
                            args_expr.push(self.eval_operand(op)?);
                        }
                        *self.memory.get_mut(place).ok_or_else(|| {
                            err_unsup_format!("Place {:?} is not allocated", place)
                        })? = Expr::Apply(Box::new(func_expr), args_expr);
                        self.block = Some(*block);
                        self.statement = 0;
                        Ok(true)
                    }
                    None => Err(err_unsup_format!("Call terminator does not assign").into()),
                }
            }
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
                        Expr::Value(Value::Const(bytes, self.transl_tykind(&switch_ty.sty)?));

                    let mut target_expr = interpreter
                        .memory
                        .get(&Local::from_usize(0).into())
                        .ok_or_else(|| err_unsup_format!("Return place is not allocated"))?
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
                        .get(&Local::from_usize(0).into())
                        .ok_or_else(|| err_unsup_format!("Return place is not allocated"))?
                        .clone(),
                );

                *self
                    .memory
                    .get_mut(&Local::from_usize(0).into())
                    .ok_or_else(|| err_unsup_format!("Return place is not allocated"))? =
                    Expr::Switch(Box::new(discr_expr), values_expr, targets_expr);

                self.block = None;
                self.statement = 0;
                Ok(false)
            }
            ref tk => Err(err_unsup_format!("TerminatorKind {:?} is not supported", tk).into()),
        }
    }

    fn eval_rvalue_into_place(
        &mut self,
        rvalue: &Rvalue<'tcx>,
        place: &Place<'tcx>,
    ) -> InterpResult<'tcx> {
        let value = match rvalue {
            Rvalue::BinaryOp(bin_op, op1, op2) => Expr::BinaryOp(
                *bin_op,
                Box::new(self.eval_operand(op1)?),
                Box::new(self.eval_operand(op2)?),
            ),
            Rvalue::Ref(_, BorrowKind::Shared, place) => self
                .memory
                .get(place)
                .ok_or_else(|| {
                    err_unsup_format!("Place {:?} in reference is not allocated", place)
                })?
                .clone(),
            Rvalue::Use(op) => self.eval_operand(op)?,
            ref rv => return Err(err_unsup_format!("Rvalue {:?} unsupported", rv).into()),
        };

        *self.memory.get_mut(place).ok_or_else(|| {
            err_unsup_format!("Place {:?} in assignment is not allocated", place)
        })? = value;

        Ok(())
    }

    fn eval_operand(&self, operand: &Operand<'tcx>) -> InterpResult<'tcx, Expr> {
        Ok(match operand {
            Operand::Move(place) | Operand::Copy(place) => self
                .memory
                .get(place)
                .ok_or_else(|| {
                    err_unsup_format!("Place {:?} in move/copy is not allocated", place)
                })?
                .clone(),

            Operand::Constant(constant) => {
                let tykind = &constant.literal.ty.sty;
                let ty = self.transl_tykind(tykind)?;
                Expr::Value(match ty {
                    Ty::Func(_) => match tykind {
                        TyKind::FnDef(def_id, _) => Value::Function(*def_id, ty),
                        _ => unreachable!(),
                    },

                    _ => match constant.literal.val {
                        ConstValue::Scalar(scalar) => Value::Const(
                            scalar
                                .to_bits(Size::from_bits(ty.size().unwrap() as u64))
                                .unwrap(),
                            ty,
                        ),
                        _ => return Err(err_unsup_format!("Unsupported ConstValue").into()),
                    },
                })
            }
        })
    }

    fn check_storages(&self, def_id: &DefId) -> (Vec<Local>, Vec<Local>) {
        let mut live = Vec::new();
        let mut dead = Vec::new();

        let mir = self.tcx.optimized_mir(*def_id);

        for block in mir.basic_blocks() {
            for statement in &block.statements {
                match statement.kind {
                    StatementKind::StorageLive(local) => live.push(local),
                    StatementKind::StorageDead(local) => dead.push(local),
                    _ => (),
                }
            }
        }

        (live, dead)
    }

    fn transl_tykind(&self, ty_kind: &TyKind<'tcx>) -> InterpResult<'tcx, Ty> {
        match ty_kind {
            TyKind::Bool => Ok(Ty::Bool),
            TyKind::Int(int_ty) => Ok(Ty::Int(int_ty.bit_width().unwrap_or(64))),
            TyKind::Uint(uint_ty) => Ok(Ty::Uint(uint_ty.bit_width().unwrap_or(64))),
            TyKind::FnDef(def_id, _) => self
                .tcx
                .optimized_mir(*def_id)
                .local_decls
                .iter()
                .map(|ld| self.transl_tykind(&ld.ty.sty))
                .collect::<InterpResult<'_, Vec<Ty>>>()
                .map(|args_ty| Ty::Func(args_ty)),
            _ => Err(err_unsup_format!("Unsupported TyKind {:?}", ty_kind).into()),
        }
    }
}
