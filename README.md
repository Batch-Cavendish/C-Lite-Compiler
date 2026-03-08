# Minimal C-Lite Compiler

A pedagogical "C-Lite" compiler built in Rust that targets **x86_64 Assembly**. This project demonstrates the core phases of a compiler: Lexical Analysis, Parsing, Semantic Analysis, and Code Generation, with a focus on ABI correctness.

## Purpose
The goal of this project is to provide a clear, readable implementation of a compiler for a subset of the C language. It demonstrates how high-level constructs (loops, recursion, boolean logic) are translated into low-level machine instructions while maintaining stack alignment and calling conventions.

## Implementation Technology
- **Language:** Rust (2024 Edition)
- **Error Handling:** `thiserror` for categorized compiler errors with position tracking.
- **Architecture:** x86_64 (Intel Syntax, System V ABI).
- **Toolchain:** Cargo for building and testing.

## Current Functions
The compiler supports a subset of C with the following features:

### 1. Language Specifications
- **Types:** `int` (32-bit signed), `bool` (`true`/`false`).
- **Control Flow:** `if-else` blocks, `while` loops, and `return` statements.
- **I/O:** Built-in `printInt(int)` primitive for output.
- **Functions:** Support for function declarations, parameters, and recursion.
- **Scope:** Support for nested block scopes and variable shadowing.

### 2. Compiler Pipeline
- **Lexer:** Custom scanner with line/column tracking for error reporting.
- **Parser:** Recursive descent parser with precedence climbing and expression-level lookahead.
- **Semantic Analyzer:** Symbol table management with nested scopes and strict type/return-type checking.
- **Code Generator:** Robust x86_64 translator featuring:
    - **ABI Compliance:** Dynamic stack alignment (16-byte) during complex expression evaluations.
    - **Variable Mapping:** Pre-pass allocation of all local variables and parameters.
    - **System V ABI:** Function calling with first 6 arguments in registers.
    - **Control Flow:** Labels and conditional jumps for loops and branches.

## Usage Methods

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/tools/install) (Cargo).
- A linker/assembler (like `gcc`) to run the generated `.s` files.

### Building the Compiler
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Compiling a Program
Create `program.clite`:
```c
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

int main() {
    printInt(factorial(5));
    return 0;
}
```

Run the compiler:
```bash
cargo run -- program.clite
```

### Running the Output
Link with a C runtime:
```c
// runtime.c
#include <stdio.h>
void printInt(int x) {
    printf("%d\n", x);
}
```
Assemble and link:
```bash
gcc program.s runtime.c -o program
./program
```

## Future Improvement Plans
- [ ] **Optimization Pass:** Constant folding and simple register allocation.
- [ ] **Data Structures:** Add support for arrays and structs.
- [ ] **Global Variables:** Static memory allocation.
- [ ] **Floating Point:** Support for `float` types and XMM registers.
- [ ] **Variadic Arguments:** Support for more than 6 function arguments.
