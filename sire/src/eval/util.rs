use std::collections::BTreeSet;

use rustc::mir::interpret::ConstValue;
use rustc::mir::visit::Visitor;
use rustc::mir::*;
use rustc::ty::{Const, ParamConst};

use crate::eval::Evaluator;
use crate::sir::Param;

#[derive(Default)]
pub struct CheckStorage {
    live: Vec<Local>,
    dead: Vec<Local>,
}

impl<'tcx> CheckStorage {
    pub fn run(body: &Body<'tcx>) -> (Vec<Local>, Vec<Local>) {
        let mut check = Self::default();
        check.visit_body(body);
        (check.live, check.dead)
    }
}

impl<'tcx> Visitor<'tcx> for CheckStorage {
    fn visit_statement(&mut self, statement: &Statement<'tcx>, _location: Location) {
        match statement.kind {
            StatementKind::StorageLive(local) => self.live.push(local),
            StatementKind::StorageDead(local) => self.dead.push(local),
            _ => (),
        }
    }
}

pub struct ExtractParams<'tcx, 'eval> {
    params: BTreeSet<Param>,
    evaluator: &'eval Evaluator<'tcx>,
}

impl<'tcx, 'eval> ExtractParams<'tcx, 'eval> {
    pub fn run(evaluator: &'eval Evaluator<'tcx>, body: &Body<'tcx>) -> Vec<Param> {
        let mut extract = ExtractParams { params: Default::default(), evaluator };
        extract.visit_body(body);
        extract.params.into_iter().collect()
    }
}

impl<'tcx, 'eval> Visitor<'tcx> for ExtractParams<'tcx, 'eval> {
    fn visit_operand(&mut self, op: &Operand<'tcx>, _location: Location) {
        match op {
            Operand::Constant(box Constant {
                literal: Const { ty, val: ConstValue::Param(ParamConst { index, .. }) },
                ..
            }) => {
                // FIXME: not all rust types are supported
                let param = Param(*index as usize, self.evaluator.transl_ty(ty).unwrap());
                self.params.insert(param);
            }
            _ => (),
        }
    }
}

pub struct CheckPanic {
    panics: bool,
}

impl<'tcx> CheckPanic {
    pub fn run(body: &Body<'tcx>) -> bool {
        let mut check = CheckPanic { panics: false };
        check.visit_body(body);
        check.panics
    }
}

impl<'tcx> Visitor<'tcx> for CheckPanic {
    fn visit_terminator_kind(&mut self, kind: &TerminatorKind<'tcx>, _location: Location) {
        if let TerminatorKind::Assert { .. } = kind {
            self.panics = true;
        }
    }
}
