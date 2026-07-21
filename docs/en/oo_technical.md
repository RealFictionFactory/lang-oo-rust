# `Ó` Language - Technical Documentation

This document describes the architecture of the "`Ó`" language interpreter written in Rust.

## 1. Architecture
The interpreter implements a classic code translation and execution pipeline:
`Source Code (String)` -> **Lexer** -> `Tokens` -> **Parser** -> `AST` -> **Interpreter** -> `Execution`

The program operates in two modes:
*   **REPL:** Interactive console launched with `cargo run`. The prompt is `ó>`. Code executes after typing `.` on an empty line.
*   **File Mode:** Launched via `cargo run -- file.oo`. Reads and executes the entire file. Supports Unix shebangs (`#!`) for native shell scripting. Returns exit code `0` on success and `1` on runtime/syntax errors.

## 2. Modules (Project Structure)

The project is divided into logical modules in the `src/` directory:

### `lexer.rs`
- Responsible for lexical analysis.
- Converts a character string (`String`) into a vector of tokens (`Vec<Token>`).
- Recognizes keywords (including `fun`, `and`, `or`, `not`, `execute`, `onError`, `until`, `in`, `match`) and operators (including `->`, `??`).
- Features "peek" logic for two-character operators (e.g., `==`, `+=`, `..`, `<=`, `->`, `??`).
- Automatically recognizes floats by checking if a dot following a number is not the start of a range `..`.
- Ignores `//` comments until the end of the line. Also ignores `#!` shebangs on the very first line of a file.

### `ast.rs`
- Contains the structural definitions of the Abstract Syntax Tree (AST).
- `Expr`: Expressions that return a value when evaluated. Includes math operations, variable access, function calls, `If` (acting as a ternary expression), `ExecuteCatch` (error handling), `Unary` (negation/logical not), `Dict` (map literals), `Match` (pattern matching), `NullCoalesce` (nullish coalescing), and `Lambda` (anonymous functions).
- `Stmt`: Statements that perform actions in the code (declarations, loops, assignments) but do not return a value. Includes `LoopIn` (array iteration), `LoopBlock` (infinite loop), and `Until` (conditional break).
- Recursive trees are wrapped in `Box<T>` due to Rust's size requirements for structs.

### `parser.rs`
- Implements *Recursive Descent Parsing*.
- Builds the AST based on the `Vec<Token>` returned by the lexer.
- Separates expression parsing into strict precedence levels: `parse_expr` (and, or, `??`) -> `parse_logic` (+, -, comparisons) -> `parse_term` (*, /, %) -> `parse_unary` (-, not) -> `parse_factor` (values, parentheses).
- Uses desugaring mechanisms: e.g., `+=` is turned into an assignment tree with addition on the fly, and string interpolation `"a {b}"` creates an internal Lexer and Parser instance to return an `Expr::Binary("a " + b)` expression.
- Context-aware parsing in `parse_factor`: if `{` is encountered while expecting a value, it is parsed as a `Dict` literal; if encountered as a statement, it's a code block.
- `if`, `execute`, and `match` are parsed as expressions, allowing them to be used inline (e.g., `var x = if ...`).
- Function and method calls are parsed "postfix" at the end of `parse_factor` using a single loop, allowing for unlimited chaining (e.g., `foo()().bar()[0]`).

### `interpreter.rs`
- Implements the execution logic on the AST.
- The core architecture relies on `Rc<RefCell<T>>` for shared mutability and memory efficiency:
  - `Environment` is wrapped in `Rc<RefCell<Environment>>`. This implements the Scope Chain pattern with a `parent: Option<Rc<RefCell<Environment>>>` field, allowing functions and closures to share state and read global variables without cloning the entire environment.
  - `Value::Array` (`Rc<RefCell<Vec<Value>>>`) and `Value::Dict` (`Rc<RefCell<HashMap<String, Value>>>`) are reference types. They are passed by reference to functions, making them mutable inside functions without cloning their contents.
- The `VarInfo` struct stores the `Value`, a `is_const` flag, and an optional `type_name`. This enables **Runtime Type Checking**: if `type_name` is specified, the interpreter validates values during declaration (`Stmt::VarDecl`, `Stmt::Let`) and assignment (`Stmt::Assign`).
- The interpreter core is "dumb" and agnostic to specific types. It contains no hard-coded methods (like `push` or `read`). It only understands AST evaluation, scope management, and control flow.
- Uses a custom error system `InterpErr` with `Return`, `Break`, `Continue`, and `Err` variants. Thanks to error propagation (`?`), control flow statements "bubble up" through nested blocks. The `Until` statement leverages this by returning `InterpErr::Break` if its condition is true.
- Introduces `eval_block_as_expr`, a helper that evaluates a block of statements and returns the value of the last expression, enabling `if`, `execute`, and `match` to act as expressions.
- `execute_function` is a public helper method that executes a `Value::Function` or `Value::Builtin` with given arguments. This allows the standard library (e.g., `.map()`, `.filter()`) to execute lambdas/closures passed from the language.
- `ExecuteCatch` catches `InterpErr::Err` (binding the message to a variable) while letting control flow errors (`Return`, `Break`, `Continue`) propagate naturally.

### `stdlib.rs`
- Acts as the language's standard library.
- Registers global built-in functions (`print`, `input`, `args`, `exit`, `shell`) and pure extension methods into the `Environment`.
- Implements extension methods for Strings (`upper`, `lower`, `trim`, `contains`, `replace`, `split`) and Arrays (`contains`, `join`, `push`, `map`, `filter`). Array methods like `map` and `filter` utilize the `Environment::execute_function` helper to run passed lambdas.
- Keeping this logic separate from `interpreter.rs` ensures the interpreter core remains lightweight and generic.

### `modules/`
- Handles external or standard library extensions loaded via the `use` keyword.
- `modules/io.rs`: Implements a `File` object using a `Dict` under the hood (storing `path` and `__type__` keys). It registers a global `file(path)` constructor and extension methods (`read`, `write`, `append`, `exists`). This design keeps the interpreter completely agnostic to file operations.

### `main.rs`
- Program entry point. Handles the CLI, reads input from the console or a file, and invokes the modules in the correct order: Lexer -> Parser -> Interpreter.
- Maps the interpreter's result to process exit codes (`0` for success, `1` for errors).

### `tests.rs`
- Unit testing module. Uses the `#[cfg(test)]` attribute.
- Conducts integration tests of the entire pipeline: Lexer -> Parser -> Interpreter, verifying results, AST structures, and correct error throwing (including logical operators, loops, dictionaries, safe dictionary access, error handling, pattern matching, closures, file I/O, and type checking). (Currently 68 passing tests).

## 3. Dependencies
The project relies entirely on the Rust standard library (`std`). It does not use any external crates.
- `std::collections::HashMap` (for environments, arrays, and dictionaries).
- `std::io` and `std::fs` (for CLI, file interactions, and I/O module).
- `std::env` (for reading startup arguments).
- `std::process` (for executing shell commands and handling exit codes).
- `std::cell::RefCell` and `std::rc::Rc` (for shared mutability and memory management).
