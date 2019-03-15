use crate::mir::*;

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Place(usize),
    Value(i32),
    Function(String),
    Apply(Box<Expr>, Vec<Expr>),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    Switch(Box<Expr>, Vec<Expr>, Vec<Expr>),
    Nil,
}

impl Expr {
    fn replace(self, target: &Self, substitution: &Self) -> Self {
        if self == *target {
            substitution.clone()
        } else {
            match self {
                Expr::Apply(e1, e2) => Expr::Apply(
                    Box::new(e1.replace(target, substitution)),
                    e2.into_iter()
                        .map(|e| e.replace(target, substitution))
                        .collect(),
                ),
                Expr::Switch(e1, e2, e3) => Expr::Switch(
                    Box::new(e1.replace(target, substitution)),
                    e2.into_iter()
                        .map(|e| e.replace(target, substitution))
                        .collect(),
                    e3.into_iter()
                        .map(|e| e.replace(target, substitution))
                        .collect(),
                ),
                Expr::BinaryOp(bin_op, e1, e2) => Expr::BinaryOp(
                    bin_op,
                    Box::new(e1.replace(target, substitution)),
                    Box::new(e2.replace(target, substitution)),
                ),
                expr => expr,
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Frame {
    locals: Vec<usize>,
    block: BlockID,
    statement: StatementID,
    function: String,
}

#[derive(Clone, Debug)]
pub struct Interpreter {
    stack: Vec<Frame>,
    state: Vec<Expr>,
    functions: HashMap<String, Function>,
    result: Expr,
}

impl<'a> Interpreter {
    pub fn new() -> Self {
        Interpreter {
            stack: Vec::new(),
            state: Vec::new(),
            functions: HashMap::new(),
            result: Expr::Nil,
        }
    }

    pub fn eval_function(&mut self, f_name: &str) -> EvalResult<Expr> {
        let mut locals = Vec::new();
        let function = self
            .functions
            .get(f_name)
            .ok_or(EvalError::new("Function does not exist"))?;
        for i in 0..function.locals() {
            locals.push(self.state.len());
            if i > 0 && i <= function.args() {
                self.state.push(Expr::Place(i));
            } else {
                self.state.push(Expr::Nil);
            }
        }
        self.stack.push(Frame {
            locals,
            block: BlockID(0),
            statement: StatementID(0),
            function: f_name.to_owned(),
        });
        self.run()?;
        Ok(self.result.clone())
    }

    pub fn run(&mut self) -> EvalResult {
        while self.step()? {}
        Ok(())
    }

    pub fn step(&mut self) -> EvalResult<bool> {
        if self.stack.is_empty() {
            return Ok(false);
        }

        let frame = self.stack.first().expect("Stack is not empty");
        let block = self
            .functions
            .get(&frame.function)
            .ok_or(EvalError::new("Function does not exist"))?
            .get_block(&frame.block)
            .ok_or(EvalError(format!("Block {:?} does not exist", frame.block)))?
            .clone();
        if block.is_statement(&frame.statement) {
            let statement = block
                .get_statement(&frame.statement)
                .ok_or(EvalError(format!(
                    "Statement {:?} does not exist",
                    frame.statement
                )))?;
            self.eval_statement(statement)?
        } else {
            let terminator = block.get_terminator();
            self.eval_terminator(terminator)?;
        }
        Ok(true)
    }

    fn eval_statement(&mut self, statement: &Statement) -> EvalResult {
        match statement {
            Statement::Assign(place, rvalue) => self.eval_rvalue_to_place(rvalue, place)?,
        };
        self.stack
            .first_mut()
            .ok_or(EvalError::new("Stack is empty"))?
            .statement
            .grow();
        Ok(())
    }

    fn eval_terminator(&mut self, terminator: &Terminator) -> EvalResult {
        match terminator {
            Terminator::Return => {
                let frame = self.stack.pop().ok_or(EvalError::new("Stack is empty"))?;
                self.result = self.state[frame.locals[0]].clone();
            }
            Terminator::Goto(block_id) => {
                let frame = self
                    .stack
                    .first_mut()
                    .ok_or(EvalError::new("Stack is empty"))?;
                frame.block = block_id.clone();
                frame.statement = StatementID(0);
            }
            Terminator::SwitchInt(op, values, blocks) => match self.eval_operand(op)? {
                Expr::Value(val) => {
                    let frame = self
                        .stack
                        .first_mut()
                        .ok_or(EvalError::new("Stack is empty"))?;
                    for (i, constant) in values.iter().enumerate() {
                        match constant {
                            Constant::Int(value) => {
                                if val == *value {
                                    frame.block = blocks[i].clone();
                                    frame.statement = StatementID(0);
                                    return Ok(());
                                }
                            }
                            _ => {
                                return Err(EvalError::new(
                                    "Cannot use function as value for switch",
                                ))
                            }
                        }
                    }
                    frame.block = blocks.last().unwrap().clone();
                    frame.statement = StatementID(0);
                }
                expr => {
                    let mut expr_values: Vec<Expr> = Vec::new();
                    for constant in values {
                        match constant {
                            Constant::Int(value) => expr_values.push(Expr::Value(*value)),
                            _ => {
                                return Err(EvalError::new(
                                    "Cannot use function as value for switch",
                                ))
                            }
                        }
                    }
                    let expr_blocks: Vec<Expr> = blocks
                        .iter()
                        .map(|block_id| {
                            let mut interp = self.clone();
                            let frame = interp.stack.first_mut().unwrap();
                            frame.block = block_id.clone();
                            frame.statement = StatementID(0);
                            interp.run().unwrap();
                            interp.result
                        })
                        .collect();
                    self.stack.pop().ok_or(EvalError::new("Stack is empty"))?;
                    self.result = Expr::Switch(Box::new(expr), expr_values, expr_blocks);
                }
            },
            Terminator::Call(func, args, ret, block_id) => {
                let locals = &self
                    .stack
                    .first()
                    .ok_or(EvalError::new("Stack is empty"))?
                    .locals;
                let address = match ret {
                    Place::Local(local) => *locals
                        .get(*local)
                        .ok_or(EvalError::new("Place is not pointing to a valid local"))?,
                };
                let mut expr_args = Vec::new();
                for arg in args {
                    expr_args.push(self.eval_operand(arg)?);
                }
                *self
                    .state
                    .get_mut(address)
                    .ok_or(EvalError::new("Invalid address"))? =
                    Expr::Apply(Box::new(self.eval_operand(func)?), expr_args);
                let frame = self
                    .stack
                    .first_mut()
                    .ok_or(EvalError::new("Stack is empty"))?;
                frame.block = block_id.clone();
                frame.statement = StatementID(0);
            }
        }
        Ok(())
    }

    fn eval_rvalue_to_place(&mut self, rvalue: &Rvalue, place: &Place) -> EvalResult {
        let locals = &self
            .stack
            .first()
            .ok_or(EvalError::new("Stack is empty"))?
            .locals;
        let address = match place {
            Place::Local(local) => *locals
                .get(*local)
                .ok_or(EvalError::new("Place is not pointing to a valid local"))?,
        };

        let value = match rvalue {
            Rvalue::BinaryOp(bin_op, op1, op2) => {
                let e1 = self.eval_operand(op1)?;
                let e2 = self.eval_operand(op2)?;
                match (e1, e2) {
                    (Expr::Value(c1), Expr::Value(c2)) => match bin_op {
                        BinOp::Add => Expr::Value(c1 + c2),
                        BinOp::Sub => Expr::Value(c1 - c2),
                        BinOp::Gt => {
                            if c1 > c2 {
                                Expr::Value(1)
                            } else {
                                Expr::Value(0)
                            }
                        }
                        BinOp::Eq => {
                            if c1 == c2 {
                                Expr::Value(1)
                            } else {
                                Expr::Value(0)
                            }
                        }
                    },
                    (e1, e2) => Expr::BinaryOp(bin_op.clone(), Box::new(e1), Box::new(e2)),
                }
            }
            Rvalue::Ref(Place::Local(local)) => {
                let address = *locals
                    .get(*local)
                    .ok_or(EvalError::new("Place is not pointing to a valid local"))?;
                self.state
                    .get(address)
                    .ok_or(EvalError::new("Invalid address"))?
                    .clone()
            }

            Rvalue::Use(constant) => match constant {
                Constant::Int(v) => Expr::Value(*v),
                _ => unimplemented!(),
            },
        };
        *self
            .state
            .get_mut(address)
            .ok_or(EvalError::new("Invalid address"))? = value;
        Ok(())
    }

    fn eval_operand(&self, operand: &Operand) -> EvalResult<Expr> {
        Ok(match operand {
            Operand::Move(Place::Local(local)) => self
                .state
                .get(*local)
                .cloned()
                .unwrap_or(Expr::Place(local.clone())),
            Operand::Constant(constant) => match constant {
                Constant::Int(v) => Expr::Value(*v),
                Constant::Fun(f) => Expr::Function(f.clone()),
            },
        })
    }

    pub fn result(&self) -> &Expr {
        &self.result
    }

    pub fn add_function(&mut self, name: &str, function: Function) {
        self.functions.insert(name.to_owned(), function);
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
