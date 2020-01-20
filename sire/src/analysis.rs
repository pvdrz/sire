use rustc::mir::*;

use crate::sir::*;

pub fn find_loop<'tcx>(mir: &'tcx Body<'tcx>) -> Option<Vec<BasicBlock>> {
    get_loop_start(mir, BasicBlock::from_u32(0), Vec::new())
}

fn get_loop_start<'tcx>(
    mir: &'tcx Body<'tcx>,
    block: BasicBlock,
    mut visited: Vec<BasicBlock>,
) -> Option<Vec<BasicBlock>> {
    match visited.iter().enumerate().find(|(_, b)| **b == block) {
        Some((i, _)) => Some(visited.split_off(i)),
        None => {
            let blk = mir.basic_blocks().get(block)?;
            visited.push(block);
            match blk.terminator().kind {
                TerminatorKind::Goto { target } => get_loop_start(mir, target, visited),
                TerminatorKind::SwitchInt { ref targets, .. } => {
                    let mut result = None;
                    for target in targets {
                        result = get_loop_start(mir, *target, visited.clone());
                        if result.is_some() {
                            break;
                        }
                    }
                    result
                }
                TerminatorKind::Call { destination: Some((_, target)), .. } => {
                    get_loop_start(mir, target, visited)
                }
                _ => None,
            }
        }
    }
}

impl Expr {
    pub fn find_datatype_instances(&self) -> Vec<Ty> {
        Instanced::find_types(self)
    }
}

#[derive(Default)]
struct Instanced {
    inner: Vec<Ty>,
}

impl Instanced {
    fn find_types(expr: &Expr) -> Vec<Ty> {
        let mut this = Self::default();
        this.visit_expr(expr);
        this.inner
    }
}

impl Visitor for Instanced {
    fn visit_expr(&mut self, expr: &Expr) {
        self.super_expr(expr);
        let ty = expr.ty();

        match ty {
            Ty::Tuple(_) => {
                if !self.inner.contains(&ty) {
                    self.inner.push(ty);
                }
            }
            _ => (),
        }
    }
}
