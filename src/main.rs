mod interpreter;
mod mir;

use crate::interpreter::*;
use crate::mir::*;

fn main() -> EvalResult {
    // fn largest(a: i32, b: i32) -> i32 {
    //     if a > b {
    //         a
    //     } else {
    //         b
    //     }
    // }
    //
    // fn largest(_1: i32, _2: i32) -> i32{
    //     let mut _0: i32;                     // return place
    //     let mut _3: bool;
    //     let mut _4: i32;
    //     let mut _5: i32;
    //
    //     bb0: {
    //         _4 = _1;                         // bb0[2]: scope 0 at src/main.rs:2:8: 2:9
    //         _5 = _2;                         // bb0[4]: scope 0 at src/main.rs:2:12: 2:13
    //         _3 = Gt(move _4, move _5);       // bb0[5]: scope 0 at src/main.rs:2:8: 2:13
    //         switchInt(move _3) -> [false: bb2, otherwise: bb1]; // bb0[8]: scope 0 at src/main.rs:2:5: 6:6
    //     }
    //
    //     bb1: {
    //         _0 = _1;                         // bb1[0]: scope 0 at src/main.rs:3:9: 3:10
    //         goto -> bb3;                     // bb1[1]: scope 0 at src/main.rs:2:5: 6:6
    //     }
    //
    //     bb2: {
    //         _0 = _2;                         // bb2[0]: scope 0 at src/main.rs:5:9: 5:10
    //         goto -> bb3;                     // bb2[1]: scope 0 at src/main.rs:2:5: 6:6
    //     }
    //
    //     bb3: {
    //         return;                          // bb3[1]: scope 0 at src/main.rs:7:2: 7:2
    //     }
    // }
    let largest = Function::new(
        6,
        2,
        vec![
            Block::new(
                vec![
                    Statement::Assign(Place::Local(4), Rvalue::Ref(Place::Local(1))),
                    Statement::Assign(Place::Local(5), Rvalue::Ref(Place::Local(2))),
                    Statement::Assign(
                        Place::Local(3),
                        Rvalue::BinaryOp(
                            BinOp::Gt,
                            Operand::Move(Place::Local(4)),
                            Operand::Move(Place::Local(5)),
                        ),
                    ),
                ],
                Terminator::SwitchInt(
                    Operand::Move(Place::Local(3)),
                    vec![Constant::Int(0)],
                    vec![BlockID(2), BlockID(1)],
                ),
            ),
            Block::new(
                vec![Statement::Assign(
                    Place::Local(0),
                    Rvalue::Ref(Place::Local(1)),
                )],
                Terminator::Goto(BlockID(3)),
            ),
            Block::new(
                vec![Statement::Assign(
                    Place::Local(0),
                    Rvalue::Ref(Place::Local(2)),
                )],
                Terminator::Goto(BlockID(3)),
            ),
            Block::new(vec![], Terminator::Return),
        ],
    );

    // fn foo(_1: i32) -> i32{
    //     let mut _0: i32;                     // return place
    //     let mut _2: bool;
    //     let mut _3: i32;
    //     let mut _4: i32;
    //     let mut _5: i32;
    //     let mut _6: i32;
    //     let mut _7: i32;
    //
    //     bb0: {
    //         _3 = _1;                         // bb0[2]: scope 0 at src/main.rs:2:8: 2:9
    //         _2 = Eq(move _3, const 0i32);    // bb0[3]: scope 0 at src/main.rs:2:8: 2:14
    //         switchInt(move _2) -> [false: bb2, otherwise: bb1]; // bb0[5]: scope 0 at src/main.rs:2:5: 6:6
    //     }
    //
    //     bb1: {
    //         _0 = const 0i32;                 // bb1[0]: scope 0 at src/main.rs:3:9: 3:10
    //         goto -> bb4;                     // bb1[1]: scope 0 at src/main.rs:2:5: 6:6
    //     }
    //
    //     bb2: {
    //         _4 = _1;                         // bb2[1]: scope 0 at src/main.rs:5:9: 5:10
    //         _7 = _1;                         // bb2[5]: scope 0 at src/main.rs:5:17: 5:18
    //         _6 = Sub(move _7, const 1i32);   // bb2[6]: scope 0 at src/main.rs:5:17: 5:22
    //         _5 = const foo(move _6) -> bb3;  // bb2[8]: scope 0 at src/main.rs:5:13: 5:23
    //     }
    //
    //     bb3: {
    //         _0 = Add(move _4, move _5);      // bb3[1]: scope 0 at src/main.rs:5:9: 5:23
    //         goto -> bb4;                     // bb3[4]: scope 0 at src/main.rs:2:5: 6:6
    //     }
    //
    //     bb4: {
    //         return;                          // bb4[1]: scope 0 at src/main.rs:7:2: 7:2
    //     }
    // }

    let foo = Function::new(
        8,
        1,
        vec![
            Block::new(
                vec![
                    Statement::Assign(Place::Local(3), Rvalue::Ref(Place::Local(1))),
                    Statement::Assign(
                        Place::Local(2),
                        Rvalue::BinaryOp(
                            BinOp::Eq,
                            Operand::Move(Place::Local(3)),
                            Operand::Constant(Constant::Int(0)),
                        ),
                    ),
                ],
                Terminator::SwitchInt(
                    Operand::Move(Place::Local(2)),
                    vec![Constant::Int(0)],
                    vec![BlockID(2), BlockID(1)],
                ),
            ),
            Block::new(
                vec![Statement::Assign(
                    Place::Local(0),
                    Rvalue::Use(Constant::Int(0)),
                )],
                Terminator::Goto(BlockID(4)),
            ),
            Block::new(
                vec![
                    Statement::Assign(Place::Local(4), Rvalue::Ref(Place::Local(1))),
                    Statement::Assign(Place::Local(7), Rvalue::Ref(Place::Local(1))),
                    Statement::Assign(
                        Place::Local(6),
                        Rvalue::BinaryOp(
                            BinOp::Sub,
                            Operand::Move(Place::Local(7)),
                            Operand::Constant(Constant::Int(1)),
                        ),
                    ),
                ],
                Terminator::Call(
                    Operand::Constant(Constant::Fun("foo".to_owned())),
                    vec![Operand::Move(Place::Local(6))],
                    Place::Local(5),
                    BlockID(3),
                ),
            ),
            Block::new(
                vec![Statement::Assign(
                    Place::Local(0),
                    Rvalue::BinaryOp(
                        BinOp::Add,
                        Operand::Move(Place::Local(4)),
                        Operand::Move(Place::Local(5)),
                    ),
                )],
                Terminator::Goto(BlockID(4)),
            ),
            Block::new(vec![], Terminator::Return),
        ],
    );

    let mut interpreter = Interpreter::new();
    interpreter.add_function("foo", foo);
    println!("foo = {:?}", interpreter.eval_function("foo"));

    Ok(())
}
