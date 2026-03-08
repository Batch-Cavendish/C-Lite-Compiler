use crate::ast::{BinaryOp, Expression, FunctionDecl, Program, Statement, Type};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SemanticError {
    #[error("Variable '{0}' not declared")]
    VariableNotDeclared(String),
    #[error("Variable '{0}' already declared in this scope")]
    VariableAlreadyDeclared(String),
    #[error("Type mismatch: expected {0:?}, found {1:?}")]
    TypeMismatch(Type, Type),
    #[error("Function '{0}' not declared")]
    FunctionNotDeclared(String),
    #[error("Function '{0}' already declared")]
    FunctionAlreadyDeclared(String),
    #[error("Argument count mismatch for function '{0}': expected {1}, found {2}")]
    ArgCountMismatch(String, usize, usize),
    #[error("Invalid operation '{0:?}' on types {1:?} and {2:?}")]
    InvalidBinaryOp(BinaryOp, Type, Type),
    #[error("PrintInt expects Int, found {0:?}")]
    InvalidPrintIntType(Type),
    #[error("Condition expects Bool, found {0:?}")]
    InvalidConditionType(Type),
    #[error("Function '{0}' must return a value of type {1:?}")]
    MissingReturnValue(String, Type),
}

struct SymbolTable {
    scopes: Vec<HashMap<String, Type>>,
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: String, ty: Type) -> Result<(), SemanticError> {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.contains_key(&name) {
            return Err(SemanticError::VariableAlreadyDeclared(name));
        }
        current_scope.insert(name, ty);
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}

struct FunctionSignature {
    return_type: Type,
    param_types: Vec<Type>,
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    functions: HashMap<String, FunctionSignature>,
    current_return_type: Option<Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            functions: HashMap::new(),
            current_return_type: None,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        for func in &program.functions {
            if self.functions.contains_key(&func.name) {
                return Err(SemanticError::FunctionAlreadyDeclared(func.name.clone()));
            }
            self.functions.insert(
                func.name.clone(),
                FunctionSignature {
                    return_type: func.return_type.clone(),
                    param_types: func.params.iter().map(|(t, _)| t.clone()).collect(),
                },
            );
        }

        for func in &program.functions {
            self.analyze_function(func)?;
        }
        Ok(())
    }

    fn analyze_function(&mut self, func: &FunctionDecl) -> Result<(), SemanticError> {
        self.current_return_type = Some(func.return_type.clone());
        self.symbol_table.enter_scope();

        for (ty, name) in &func.params {
            self.symbol_table.define(name.clone(), ty.clone())?;
        }

        self.analyze_statement(&func.body)?;

        self.symbol_table.exit_scope();
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), SemanticError> {
        match stmt {
            Statement::VarDecl(ty, name, init) => {
                if let Some(expr) = init {
                    let expr_ty = self.analyze_expression(expr)?;
                    if &expr_ty != ty {
                        return Err(SemanticError::TypeMismatch(ty.clone(), expr_ty));
                    }
                }
                self.symbol_table.define(name.clone(), ty.clone())?;
            }
            Statement::Assign(name, expr) => {
                let var_ty = self
                    .symbol_table
                    .lookup(name)
                    .cloned()
                    .ok_or_else(|| SemanticError::VariableNotDeclared(name.clone()))?;
                let expr_ty = self.analyze_expression(expr)?;
                if var_ty != expr_ty {
                    return Err(SemanticError::TypeMismatch(var_ty, expr_ty));
                }
            }
            Statement::Block(stmts) => {
                self.symbol_table.enter_scope();
                for s in stmts {
                    self.analyze_statement(s)?;
                }
                self.symbol_table.exit_scope();
            }
            Statement::If(cond, then, els) => {
                let cond_ty = self.analyze_expression(cond)?;
                if cond_ty != Type::Bool {
                    return Err(SemanticError::InvalidConditionType(cond_ty));
                }
                self.analyze_statement(then)?;
                if let Some(e) = els {
                    self.analyze_statement(e)?;
                }
            }
            Statement::While(cond, body) => {
                let cond_ty = self.analyze_expression(cond)?;
                if cond_ty != Type::Bool {
                    return Err(SemanticError::InvalidConditionType(cond_ty));
                }
                self.analyze_statement(body)?;
            }
            Statement::Return(expr_opt) => {
                let expected = self.current_return_type.as_ref().unwrap().clone();
                match expr_opt {
                    Some(expr) => {
                        let actual = self.analyze_expression(expr)?;
                        if actual != expected {
                            return Err(SemanticError::TypeMismatch(expected, actual));
                        }
                    }
                    None => {
                        return Err(SemanticError::TypeMismatch(expected, Type::Bool));
                    }
                }
            }
            Statement::PrintInt(expr) => {
                let ty = self.analyze_expression(expr)?;
                if ty != Type::Int {
                    return Err(SemanticError::InvalidPrintIntType(ty));
                }
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<Type, SemanticError> {
        match expr {
            Expression::IntLiteral(_) => Ok(Type::Int),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            Expression::Variable(name) => self
                .symbol_table
                .lookup(name)
                .cloned()
                .ok_or_else(|| SemanticError::VariableNotDeclared(name.clone())),
            Expression::Binary(op, left, right) => {
                let l_ty = self.analyze_expression(left)?;
                let r_ty = self.analyze_expression(right)?;
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(SemanticError::InvalidBinaryOp(op.clone(), l_ty, r_ty))
                        }
                    }
                    BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                        if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::InvalidBinaryOp(op.clone(), l_ty, r_ty))
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if l_ty == r_ty {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::InvalidBinaryOp(op.clone(), l_ty, r_ty))
                        }
                    }
                }
            }
            Expression::Call(name, args) => {
                let (param_types, return_type) = if let Some(sig) = self.functions.get(name) {
                    (sig.param_types.clone(), sig.return_type.clone())
                } else {
                    return Err(SemanticError::FunctionNotDeclared(name.clone()));
                };

                if args.len() != param_types.len() {
                    return Err(SemanticError::ArgCountMismatch(
                        name.clone(),
                        param_types.len(),
                        args.len(),
                    ));
                }
                for (arg, expected) in args.iter().zip(&param_types) {
                    let actual = self.analyze_expression(arg)?;
                    if &actual != expected {
                        return Err(SemanticError::TypeMismatch(expected.clone(), actual));
                    }
                }
                Ok(return_type)
            }
        }
    }
}
