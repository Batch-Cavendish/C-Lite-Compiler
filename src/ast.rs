#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    IntLiteral(i32),
    BoolLiteral(bool),
    Variable(String),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Call(String, Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    VarDecl(Type, String, Option<Expression>),
    Assign(String, Expression),
    Block(Vec<Statement>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    Return(Option<Expression>),
    PrintInt(Expression),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub return_type: Type,
    pub params: Vec<(Type, String)>,
    pub body: Statement, // Should be a Statement::Block
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub functions: Vec<FunctionDecl>,
}
