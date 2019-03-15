use crate::mir::*;

#[derive(Clone, Debug)]
pub enum Expr {
    Place(usize),
    Value(i32),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    Switch(Box<Expr>, Vec<Expr>, Vec<Expr>),
    Nil,
}

#[derive(Clone, Debug)]
struct Frame<'a> {
    locals: Vec<usize>,
    block: BlockID,
    statement: StatementID,
    function: &'a Function,
}

#[derive(Clone, Debug)]
pub struct Interpreter<'a> {
    stack: Vec<Frame<'a>>,
    state: Vec<Expr>,
    result: Expr,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Interpreter {
            stack: Vec::new(),
            state: Vec::new(),
            result: Expr::Nil,
        }
    }

    pub fn eval_function(&mut self, function: &'a Function) -> EvalResult<Expr> {
        let mut locals = Vec::new();
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
            function,
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
        let block = frame
            .function
            .get_block(&frame.block)
            .ok_or(EvalError(format!("Block {:?} does not exist", frame.block)))?;
        if block.is_statement(&frame.statement) {
            let statement = block
                .get_statement(&frame.statement)
                .ok_or(EvalError(format!(
                    "Statement {:?} does not exist",
                    frame.statement
                )))?;
            self.eval_statement(statement)?;
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
                    for (i, Constant(value)) in values.iter().enumerate() {
                        if val == *value {
                            frame.block = blocks[i].clone();
                            frame.statement = StatementID(0);
                            return Ok(());
                        }
                    }
                    frame.block = blocks.last().unwrap().clone();
                    frame.statement = StatementID(0);
                }
                expr => {
                    let expr_values: Vec<Expr> =
                        values.iter().map(|Constant(v)| Expr::Value(*v)).collect();
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

            Rvalue::Use(Constant(v)) => Expr::Value(*v),
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
            Operand::Constant(Constant(v)) => Expr::Value(*v),
        })
    }

    pub fn result(&self) -> &Expr {
        &self.result
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
