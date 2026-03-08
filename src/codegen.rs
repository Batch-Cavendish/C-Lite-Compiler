use crate::ast::{BinaryOp, Expression, FunctionDecl, Program, Statement};
use std::collections::HashMap;

pub struct CodeGenerator {
    output: String,
    label_counter: usize,
    current_ret_label: String,
    var_map: HashMap<String, i32>, // Name -> RBP offset
    stack_depth: usize, // Track temporary pushes
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            output: String::new(),
            label_counter: 0,
            current_ret_label: String::new(),
            var_map: HashMap::new(),
            stack_depth: 0,
        }
    }

    fn push_rax(&mut self) {
        self.emit("push rax");
        self.stack_depth += 8;
    }

    fn pop_reg(&mut self, reg: &str) {
        self.emit(&format!("pop {}", reg));
        self.stack_depth -= 8;
    }

    fn call_aligned(&mut self, func: &str) {
        // System V ABI requires 16-byte alignment at call
        // After prologue, stack is 16-byte aligned.
        // If stack_depth is not a multiple of 16, we need to adjust.
        let needs_alignment = (self.stack_depth % 16) != 0;
        if needs_alignment {
            self.emit("sub rsp, 8");
        }
        self.emit(&format!("call {}", func));
        if needs_alignment {
            self.emit("add rsp, 8");
        }
    }

    fn next_label(&mut self) -> String {
        let label = format!(".L{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str("  ");
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn label(&mut self, label: &str) {
        self.output.push_str(label);
        self.output.push_str(":\n");
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.output.push_str(".intel_syntax noprefix\n");
        self.output.push_str(".global main\n");
        self.output.push_str(".extern printInt\n\n");

        for func in &program.functions {
            self.gen_function(func);
        }
        self.output.clone()
    }

    fn gen_function(&mut self, func: &FunctionDecl) {
        self.label(&func.name);
        let ret_label = self.next_label();
        self.current_ret_label = ret_label.clone();

        // Prologue
        self.emit("push rbp");
        self.emit("mov rbp, rsp");

        // Map variables to offsets
        self.var_map.clear();
        let mut offset = 0;
        
        // Params first
        for (_, name) in &func.params {
            offset += 8;
            self.var_map.insert(name.clone(), -offset);
        }
        
        // Then locals
        self.collect_locals(&func.body, &mut offset);

        // Stack alignment (16-byte)
        let stack_size = (offset + 15) & !15;
        if stack_size > 0 {
            self.emit(&format!("sub rsp, {}", stack_size));
        }

        // Move params from registers to stack
        let param_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        for (i, (_, name)) in func.params.iter().enumerate() {
            if i < 6 {
                let off = self.var_map.get(name).unwrap();
                self.emit(&format!("mov [rbp {:+1}], {}", off, param_regs[i]));
            }
        }

        self.gen_statement(&func.body);

        // Epilogue
        self.label(&ret_label);
        self.emit("mov rsp, rbp");
        self.emit("pop rbp");
        self.emit("ret");
        self.output.push('\n');
    }

    fn collect_locals(&mut self, stmt: &Statement, offset: &mut i32) {
        match stmt {
            Statement::VarDecl(_, name, _) => {
                if !self.var_map.contains_key(name) {
                    *offset += 8;
                    self.var_map.insert(name.clone(), -*offset);
                }
            }
            Statement::Block(stmts) => {
                for s in stmts {
                    self.collect_locals(s, offset);
                }
            }
            Statement::If(_, then, els) => {
                self.collect_locals(then, offset);
                if let Some(e) = els {
                    self.collect_locals(e, offset);
                }
            }
            Statement::While(_, body) => {
                self.collect_locals(body, offset);
            }
            _ => {}
        }
    }

    fn gen_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl(_, name, init) => {
                if let Some(expr) = init {
                    self.gen_expression(expr);
                    let off = self.var_map.get(name).unwrap();
                    self.emit(&format!("mov [rbp {:+1}], rax", off));
                }
            }
            Statement::Assign(name, expr) => {
                self.gen_expression(expr);
                let off = self.var_map.get(name).expect("Variable not found");
                self.emit(&format!("mov [rbp {:+1}], rax", off));
            }
            Statement::Block(stmts) => {
                for s in stmts {
                    self.gen_statement(s);
                }
            }
            Statement::If(cond, then, els) => {
                let else_label = self.next_label();
                let end_label = self.next_label();

                self.gen_expression(cond);
                self.emit("cmp rax, 0");
                self.emit(&format!("je {}", else_label));

                self.gen_statement(then);
                self.emit(&format!("jmp {}", end_label));

                self.label(&else_label);
                if let Some(e) = els {
                    self.gen_statement(e);
                }
                self.label(&end_label);
            }
            Statement::While(cond, body) => {
                let start_label = self.next_label();
                let end_label = self.next_label();

                self.label(&start_label);
                self.gen_expression(cond);
                self.emit("cmp rax, 0");
                self.emit(&format!("je {}", end_label));

                self.gen_statement(body);
                self.emit(&format!("jmp {}", start_label));

                self.label(&end_label);
            }
            Statement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.gen_expression(expr);
                }
                self.emit(&format!("jmp {}", self.current_ret_label));
            }
            Statement::PrintInt(expr) => {
                self.gen_expression(expr);
                self.emit("mov rdi, rax");
                self.call_aligned("printInt");
            }
            Statement::Expression(expr) => {
                self.gen_expression(expr);
            }
        }
    }

    fn gen_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntLiteral(i) => {
                self.emit(&format!("mov rax, {}", i));
            }
            Expression::BoolLiteral(b) => {
                self.emit(&format!("mov rax, {}", if *b { 1 } else { 0 }));
            }
            Expression::Variable(name) => {
                let off = self.var_map.get(name).expect("Variable not found");
                self.emit(&format!("mov rax, [rbp {:+1}]", off));
            }
            Expression::Binary(op, left, right) => {
                self.gen_expression(left);
                self.push_rax();
                self.gen_expression(right);
                self.emit("mov rdi, rax");
                self.pop_reg("rax");

                match op {
                    BinaryOp::Add => self.emit("add rax, rdi"),
                    BinaryOp::Sub => self.emit("sub rax, rdi"),
                    BinaryOp::Mul => self.emit("imul rax, rdi"),
                    BinaryOp::Div => {
                        self.emit("cqo");
                        self.emit("idiv rdi");
                    }
                    BinaryOp::Equal => {
                        self.emit("cmp rax, rdi");
                        self.emit("sete al");
                        self.emit("movzx rax, al");
                    }
                    BinaryOp::NotEqual => {
                        self.emit("cmp rax, rdi");
                        self.emit("setne al");
                        self.emit("movzx rax, al");
                    }
                    BinaryOp::Less => {
                        self.emit("cmp rax, rdi");
                        self.emit("setl al");
                        self.emit("movzx rax, al");
                    }
                    BinaryOp::LessEqual => {
                        self.emit("cmp rax, rdi");
                        self.emit("setle al");
                        self.emit("movzx rax, al");
                    }
                    BinaryOp::Greater => {
                        self.emit("cmp rax, rdi");
                        self.emit("setg al");
                        self.emit("movzx rax, al");
                    }
                    BinaryOp::GreaterEqual => {
                        self.emit("cmp rax, rdi");
                        self.emit("setge al");
                        self.emit("movzx rax, al");
                    }
                }
            }
            Expression::Call(name, args) => {
                let regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                
                for arg in args {
                    self.gen_expression(arg);
                    self.push_rax();
                }

                for i in (0..args.len().min(6)).rev() {
                    self.pop_reg(regs[i]);
                }
                
                self.call_aligned(name);
            }
        }
    }
}
