pub mod lexer;
pub mod ast;
pub mod parser;
pub mod semantic_analyzer;
pub mod codegen;

#[cfg(test)]
mod tests {
    use super::lexer::Lexer;
    use super::parser::Parser;
    use super::semantic_analyzer::SemanticAnalyzer;
    use super::codegen::CodeGenerator;

    #[test]
    fn test_full_pipeline() {
        let input = r#"
            int factorial(int n) {
                if (n <= 1) return 1;
                return n * factorial(n - 1);
            }

            int main() {
                printInt(factorial(5));
                return 0;
            }
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&program).unwrap();

        let mut codegen = CodeGenerator::new();
        let asm = codegen.generate(&program);

        assert!(asm.contains("factorial:"));
        assert!(asm.contains("main:"));
        assert!(asm.contains("call factorial"));
        assert!(asm.contains("call printInt"));
    }

    #[test]
    fn test_boolean_logic() {
        let input = r#"
            int main() {
                bool b = true;
                if (b == false) {
                    printInt(0);
                } else {
                    printInt(1);
                }
                return 0;
            }
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&program).unwrap();

        let mut codegen = CodeGenerator::new();
        let _asm = codegen.generate(&program);
    }
}
