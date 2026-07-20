# Ó Language - Technical Documentation

This document describes the architecture of the "Ó" language interpreter written in Rust.

## 1. Architecture
The interpreter implements a classic code translation and execution pipeline:
`Source Code (String)` -> **Lexer** -> `Tokens` -> **Parser** -> `AST` -> **Interpreter** -> `Execution`

The program operates in two modes:
*   **REPL:** Interactive console launched with `cargo run`. The prompt is `ó>`. Code executes after typing `.` on an empty line.
*   **File Mode:** Launched via `cargo run -- file.oo`. Reads and executes the entire file.

## 2. Modules (Project Structure)

The project is divided into logical modules in the `src/` directory:

### `lexer.rs`
- Responsible for lexical analysis.
- Converts a character string (`String`) into a vector of tokens (`Vec<Token>`).
- Recognizes keywords, literals (Number, Decimal, String, Bool), identifiers, and operators.
- Features "peek" logic for two-character operators (e.g., `==`, `+=`, `..`, `<=`).
- Automatically recognizes floats by checking if a dot following a number is not the start of a range `..`.
- Ignores `//` comments until the end of the line, generating `NewLine` tokens that signify the end of a statement.

### `ast.rs`
- Contains the structural definitions of the Abstract Syntax Tree (AST).
- `Expr`: Expressions that return a value when evaluated (e.g., math operations, variable access, function calls).
- `Stmt`: Statements that perform actions in the code (declarations, loops, assignments) but do not return a value.
- Recursive trees are wrapped in `Box<T>` due to Rust's size requirements for structs.

### `parser.rs`
- Implements *Recursive Descent Parsing*.
- Builds the AST based on the `Vec<Token>` returned by the lexer.
- Separates expression parsing into precedence levels: `parse_expr` (+, -, comparisons) -> `parse_term` (*, /, %) -> `parse_factor` (values, parentheses).
- Uses desugaring mechanisms: e.g., `+=` is turned into an assignment tree with addition on the fly, and string interpolation `"a {b}"` creates an internal Lexer and Parser instance to return an `Expr::Binary("a " + b)` expression.
- Function and method calls are parsed "postfix" at the end of `parse_factor` using a single loop, allowing for unlimited chaining (e.g., `foo()().bar()[0]`).

### `interpreter.rs`
- Implements the execution logic on the AST.
- The main structure is `Environment` (which replaced a flat `HashMap`), implementing the Scope Chain pattern with a `parent: Option<Box<Environment>>` field. This allows functions to read global variables without permanently overwriting them with their local ones.
- Uses a custom error system `InterpErr` with `Return`, `Break`, and `Continue` variants. Thanks to error propagation (`?`), these statements "bubble up" through nested blocks until they reach a loop or function body, elegantly solving control flow without boolean flags.
- Built-in functions (`print`, `input`) are stored as pointers to Rust functions (`BuiltinFn`) inside the `Value` enum.
- Extension methods (e.g., `asNumber`, `upper`) are stored in a separate `extensions` map within the `Environment` and are looked up dynamically during `MethodCall` evaluation. Mutating methods (like `push`) are handled directly in the interpreter to access environment references.

### `stdlib.rs`
- Acts as the language's standard library.
- Registers global built-in functions and extension methods into the `Environment`.
- Keeping this logic separate from `interpreter.rs` ensures the interpreter core remains lightweight and generic.

### `modules/`
- Handles external or standard library extensions loaded via the `use` keyword (e.g., `use io`).
- Currently serves as a placeholder for future I/O operations and other modules.

### `main.rs`
- Program entry point. Handles the CLI, reads input from the console or a file, and invokes the modules in the correct order: Lexer -> Parser -> Interpreter.

### `tests.rs`
- Unit testing module. Uses the `#[cfg(test)]` attribute.
- Conducts integration tests of the entire pipeline: Lexer -> Parser -> Interpreter, verifying results and correct error throwing. (Currently 33 passing tests).

## 3. Dependencies
The project relies entirely on the Rust standard library (`std`). It does not use any external crates.
- `std::collections::HashMap` (for environments and arrays).
- `std::io` and `std::fs` (for CLI and file interactions).
- `std::env` (for reading startup arguments).
