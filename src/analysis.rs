use rustc::mir::*;

pub fn find_loop<'tcx>(mir: &'tcx Mir<'tcx>) -> Option<Vec<BasicBlock>> {
    get_loop_start(mir, BasicBlock::from_u32(0), Vec::new())
}

fn get_loop_start<'tcx>(
    mir: &'tcx Mir<'tcx>,
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
                _ => None,
            }
        }
    }
}
