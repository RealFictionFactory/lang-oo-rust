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
- Recognizes keywords (including `and`, `or`, `not`, `execute`, `onError`, `until`, `in`), literals (Number, Decimal, String, Bool), identifiers, and operators.
- Features "peek" logic for two-character operators (e.g., `==`, `+=`, `..`, `<=`).
- Automatically recognizes floats by checking if a dot following a number is not the start of a range `..`.
- Ignores `//` comments until the end of the line, generating `NewLine` tokens that signify the end of a statement.

### `ast.rs`
- Contains the structural definitions of the Abstract Syntax Tree (AST).
- `Expr`: Expressions that return a value when evaluated. Includes math operations, variable access, function calls, `If` (acting as a ternary expression), `ExecuteCatch` (error handling), `Unary` (negation/logical not), and `Dict` (map literals).
- `Stmt`: Statements that perform actions in the code (declarations, loops, assignments) but do not return a value. Includes `LoopIn` (array iteration), `LoopBlock` (infinite loop), and `Until` (conditional break).
- Recursive trees are wrapped in `Box<T>` due to Rust's size requirements for structs.

### `parser.rs`
- Implements *Recursive Descent Parsing*.
- Builds the AST based on the `Vec<Token>` returned by the lexer.
- Separates expression parsing into strict precedence levels: `parse_expr` (and, or) -> `parse_logic` (+, -, comparisons) -> `parse_term` (*, /, %) -> `parse_unary` (-, not) -> `parse_factor` (values, parentheses).
- Uses desugaring mechanisms: e.g., `+=` is turned into an assignment tree with addition on the fly, and string interpolation `"a {b}"` creates an internal Lexer and Parser instance to return an `Expr::Binary("a " + b)` expression.
- Context-aware parsing in `parse_factor`: if `{` is encountered while expecting a value, it is parsed as a `Dict` literal; if encountered as a statement, it's a code block.
- `if` and `execute` are parsed as expressions, allowing them to be used inline (e.g., `var x = if ...`).
- Function and method calls are parsed "postfix" at the end of `parse_factor` using a single loop, allowing for unlimited chaining (e.g., `foo()().bar()[0]`).

### `interpreter.rs`
- Implements the execution logic on the AST.
- The main structure is `Environment`, implementing the Scope Chain pattern with a `parent: Option<Box<Environment>>` field. This allows functions to read global variables without permanently overwriting them with their local ones.
- Uses a custom error system `InterpErr` with `Return`, `Break`, `Continue`, and `Err` variants. Thanks to error propagation (`?`), control flow statements "bubble up" through nested blocks. The `Until` statement leverages this by returning `InterpErr::Break` if its condition is true.
- Introduces `eval_block_as_expr`, a helper that evaluates a block of statements and returns the value of the last expression, enabling `if` and `execute` to act as expressions.
- Implements short-circuit evaluation for logical operators (`and`, `or`).
- `ExecuteCatch` catches `InterpErr::Err` (binding the message to a variable) while letting control flow errors (`Return`, `Break`, `Continue`) propagate naturally.
- Built-in functions (`print`, `input`) are stored as pointers to Rust functions (`BuiltinFn`) inside the `Value` enum.
- Extension methods (e.g., `asNumber`, `upper`) are stored in a separate `extensions` map within the `Environment` and are looked up dynamically during `MethodCall` evaluation. Mutating methods (like `push` for arrays or index assignment for dicts) are handled directly in the interpreter to access environment references.

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
- Conducts integration tests of the entire pipeline: Lexer -> Parser -> Interpreter, verifying results, AST structures, and correct error throwing (including logical operators, loops, dictionaries, and error handling). (Currently 49 passing tests).

## 3. Dependencies
The project relies entirely on the Rust standard library (`std`). It does not use any external crates.
- `std::collections::HashMap` (for environments, arrays, and dictionaries).
- `std::io` and `std::fs` (for CLI and file interactions).
- `std::env` (for reading startup arguments).
