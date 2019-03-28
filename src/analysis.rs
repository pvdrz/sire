use crate::mir::*;

pub fn find_loop(fun: &Function) -> Option<Vec<BlockID>> {
    get_loop_start(fun, &BlockID(0), Vec::new())
}

fn get_loop_start(
    fun: &Function,
    block_id: &BlockID,
    mut visited: Vec<BlockID>,
) -> Option<Vec<BlockID>> {
    match visited.iter().enumerate().find(|(_, b)| **b == *block_id) {
        Some((i, _)) => Some(visited.split_off(i)),
        None => {
            let block = fun.get_block(block_id)?;
            visited.push(block_id.clone());
            match block.get_terminator() {
                Terminator::Goto(next_id) => get_loop_start(fun, next_id, visited),
                Terminator::SwitchInt(_, _, next_ids) => {
                    let mut result = None;
                    for next_id in next_ids {
                        result = get_loop_start(fun, next_id, visited.clone());
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
