#[derive(Clone, Debug)]
pub struct BlockID(pub usize);

#[derive(Clone, Debug)]
pub struct StatementID(pub usize);

impl StatementID {
    pub fn grow(&mut self) {
        *self = StatementID(self.0 + 1)
    }
}

#[derive(Clone, Debug)]
pub enum Place {
    // Static(usize),
    Local(usize),
    // Promoted(usize)
    // Deref(Box<Place>),
    // Field(Box<Place>, usize),
    // Index(Box<Place>, usize, usize)
}

#[derive(Clone, Debug)]
pub enum Constant {
    Int(i32),
    Fun(String),
}

#[derive(Clone, Debug)]
pub enum Operand {
    // Copy(Place),
    Move(Place),
    Constant(Constant),
}

#[derive(Clone, Debug)]
pub enum Rvalue {
    Use(Constant),
    Ref(Place),
    // NullaryOp(NullOp)
    // UnaryOp(UnOp, Operand),
    BinaryOp(BinOp, Operand, Operand),
    // Len(Place),
    // Discriminant
    // Aggregate(Vec<Operand>)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Gt,
    Eq,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Assign(Place, Rvalue),
    // SetDiscriminant(Place, usize),
    // StorageLive(usize),
    // StorageDead(usize),
    // Nop
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Return,
    Goto(BlockID),
    SwitchInt(Operand, Vec<Constant>, Vec<BlockID>),
    Call(Operand, Vec<Operand>, Place, BlockID),
}

#[derive(Clone, Debug)]
pub struct Block {
    statements: Vec<Statement>,
    terminator: Terminator,
}

impl Block {
    pub fn new(statements: Vec<Statement>, terminator: Terminator) -> Self {
        Block {
            statements,
            terminator,
        }
    }

    pub fn is_statement(&self, StatementID(id): &StatementID) -> bool {
        *id < self.statements.len()
    }

    pub fn get_statement(&self, StatementID(id): &StatementID) -> Option<&Statement> {
        self.statements.get(*id)
    }

    pub fn get_terminator(&self) -> &Terminator {
        &self.terminator
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    locals: usize,
    args: usize,
    blocks: Vec<Block>,
}

impl Function {
    pub fn new(locals: usize, args: usize, blocks: Vec<Block>) -> Self {
        Function {
            locals,
            args,
            blocks,
        }
    }

    pub fn get_block(&self, BlockID(id): &BlockID) -> Option<&Block> {
        self.blocks.get(*id)
    }

    pub fn locals(&self) -> usize {
        self.locals
    }

    pub fn args(&self) -> usize {
        self.args
    }
}
