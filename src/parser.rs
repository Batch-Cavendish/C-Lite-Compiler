use crate::ast::{BinaryOp, Expression, FunctionDecl, Program, Statement, Type};
use crate::lexer::{Lexer, LexerError, Token};
use std::iter::Peekable;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexerError),
    #[error("Unexpected token: {0}, expected {1}")]
    UnexpectedToken(Token, String),
    #[error("Unexpected end of input")]
    UnexpectedEOF,
}

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser {
            lexer: lexer.peekable(),
        }
    }

    fn advance(&mut self) -> Result<Token, ParserError> {
        match self.lexer.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(e)) => Err(ParserError::LexerError(e)),
            None => Err(ParserError::UnexpectedEOF),
        }
    }

    fn peek(&mut self) -> Result<&Token, ParserError> {
        // First check if it's already an error in the peekable's buffer
        if let Some(Err(_)) = self.lexer.peek() {
            // If it is, we MUST consume it to return the error
             let err = match self.lexer.next().unwrap() {
                Err(e) => ParserError::LexerError(e),
                _ => unreachable!(),
            };
            return Err(err);
        }
        
        match self.lexer.peek() {
            Some(Ok(token)) => Ok(token),
            None => Ok(&Token::EOF),
            _ => unreachable!(), // Handled above
        }
    }

    fn expect(&mut self, expected: Token) -> Result<Token, ParserError> {
        let token = self.advance()?;
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            Ok(token)
        } else {
            Err(ParserError::UnexpectedToken(token, format!("{:?}", expected)))
        }
    }

    fn check(&mut self, expected: &Token) -> bool {
        match self.peek() {
            Ok(token) => std::mem::discriminant(token) == std::mem::discriminant(expected),
            Err(_) => false,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let mut functions = Vec::new();
        while !self.check(&Token::EOF) {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, ParserError> {
        let return_type = self.parse_type()?;
        let name = match self.advance()? {
            Token::Identifier(s) => s,
            t => return Err(ParserError::UnexpectedToken(t, "identifier".to_string())),
        };

        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let ty = self.parse_type()?;
                let p_name = match self.advance()? {
                    Token::Identifier(s) => s,
                    t => return Err(ParserError::UnexpectedToken(t, "identifier".to_string())),
                };
                params.push((ty, p_name));
                if self.check(&Token::Comma) {
                    self.advance()?;
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;

        let body = Statement::Block(self.parse_block()?);

        Ok(FunctionDecl {
            name,
            return_type,
            params,
            body,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        let token = self.advance()?;
        match token {
            Token::Int => Ok(Type::Int),
            Token::Bool => Ok(Type::Bool),
            t => Err(ParserError::UnexpectedToken(t, "type (int, bool)".to_string())),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParserError> {
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            statements.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.peek()? {
            Token::Int | Token::Bool => self.parse_var_decl(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Return => self.parse_return(),
            Token::PrintInt => self.parse_print_int(),
            Token::LBrace => Ok(Statement::Block(self.parse_block()?)),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_var_decl(&mut self) -> Result<Statement, ParserError> {
        let ty = self.parse_type()?;
        let name = match self.advance()? {
            Token::Identifier(s) => s,
            t => return Err(ParserError::UnexpectedToken(t, "identifier".to_string())),
        };

        let init = if self.check(&Token::Assign) {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(Token::Semi)?;
        Ok(Statement::VarDecl(ty, name, init))
    }

    fn parse_if(&mut self) -> Result<Statement, ParserError> {
        self.advance()?; // if
        self.expect(Token::LParen)?;
        let cond = self.parse_expression()?;
        self.expect(Token::RParen)?;
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.check(&Token::Else) {
            self.advance()?;
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Statement::If(cond, then_branch, else_branch))
    }

    fn parse_while(&mut self) -> Result<Statement, ParserError> {
        self.advance()?; // while
        self.expect(Token::LParen)?;
        let cond = self.parse_expression()?;
        self.expect(Token::RParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::While(cond, body))
    }

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        self.advance()?; // return
        let expr = if !self.check(&Token::Semi) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(Token::Semi)?;
        Ok(Statement::Return(expr))
    }

    fn parse_print_int(&mut self) -> Result<Statement, ParserError> {
        self.advance()?; // printInt
        self.expect(Token::LParen)?;
        let expr = self.parse_expression()?;
        self.expect(Token::RParen)?;
        self.expect(Token::Semi)?;
        Ok(Statement::PrintInt(expr))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParserError> {
        // Special case for assignment to avoid complex lookahead
        if let Token::Identifier(name) = self.peek()? {
            let name = name.clone();
            let mut clone = self.lexer.clone();
            clone.next(); // consume identifier
            let is_assign = match clone.peek() {
                Some(Ok(Token::Assign)) => true,
                _ => false,
            };

            if is_assign {
                self.advance()?; // consume identifier
                self.advance()?; // consume '='
                let expr = self.parse_expression()?;
                self.expect(Token::Semi)?;
                return Ok(Statement::Assign(name, expr));
            }
        }

        let expr = self.parse_expression()?;
        self.expect(Token::Semi)?;
        Ok(Statement::Expression(expr))
    }

    // --- Expression Parsing ---

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = match self.peek()? {
                Token::Eq => BinaryOp::Equal,
                Token::Ne => BinaryOp::NotEqual,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_comparison()?;
            expr = Expression::Binary(op, Box::new(expr), Box::new(right));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_term()?;
        loop {
            let op = match self.peek()? {
                Token::Lt => BinaryOp::Less,
                Token::Le => BinaryOp::LessEqual,
                Token::Gt => BinaryOp::Greater,
                Token::Ge => BinaryOp::GreaterEqual,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_term()?;
            expr = Expression::Binary(op, Box::new(expr), Box::new(right));
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_factor()?;
        loop {
            let op = match self.peek()? {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_factor()?;
            expr = Expression::Binary(op, Box::new(expr), Box::new(right));
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_primary()?;
        loop {
            let op = match self.peek()? {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_primary()?;
            expr = Expression::Binary(op, Box::new(expr), Box::new(right));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        let token = self.advance()?;
        match token {
            Token::IntLiteral(i) => Ok(Expression::IntLiteral(i)),
            Token::True => Ok(Expression::BoolLiteral(true)),
            Token::False => Ok(Expression::BoolLiteral(false)),
            Token::Identifier(name) => {
                if self.check(&Token::LParen) {
                    self.advance()?; // (
                    let mut args = Vec::new();
                    if !self.check(&Token::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.check(&Token::Comma) {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expression::Call(name, args))
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Token::LParen => {
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            t => Err(ParserError::UnexpectedToken(t, "expression".to_string())),
        }
    }
}
