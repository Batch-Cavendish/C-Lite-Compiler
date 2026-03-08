use std::env;
use std::fs;
use std::path::Path;
use std::process;

use minimal_c_compiler::codegen::CodeGenerator;
use minimal_c_compiler::lexer::Lexer;
use minimal_c_compiler::parser::Parser;
use minimal_c_compiler::semantic_analyzer::SemanticAnalyzer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename.clite>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let source_code = match fs::read_to_string(filename) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };

    // 1. Lexing
    let lexer = Lexer::new(&source_code);

    // 2. Parsing
    let mut parser = Parser::new(lexer);
    let program = match parser.parse_program() {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("Parser Error: {}", e);
            process::exit(1);
        }
    };

    // 3. Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&program) {
        eprintln!("Semantic Error: {}", e);
        process::exit(1);
    }

    // 4. Code Generation
    let mut codegen = CodeGenerator::new();
    let assembly = codegen.generate(&program);

    // 5. Output
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let output_filename = format!("{}.s", stem);

    if let Err(e) = fs::write(&output_filename, assembly) {
        eprintln!("Error writing output file: {}", e);
        process::exit(1);
    }

    println!(
        "Compilation successful! Output written to {}",
        output_filename
    );
}
