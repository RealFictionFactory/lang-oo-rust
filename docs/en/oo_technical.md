# `Ó` Language - Technical Documentation

This document describes the architecture of the "`Ó`" language interpreter written in Rust.

## 1. Architecture
The interpreter implements a classic code translation and execution pipeline:
`Source Code (String)` -> **Lexer** -> `Tokens` -> **Parser** -> `AST` -> **Interpreter** -> `Execution`

The program operates in two modes:
*   **REPL:** Interactive console launched with `cargo run`. The prompt is `ó>`. Code executes after typing `.` on an empty line.
*   **File Mode:** Launched via `cargo run -- file.oo`. Reads and executes the entire file. Supports Unix shebangs (`#!`) for native shell scripting. Returns exit code `0` on success and `1` on runtime/syntax errors.

The REPL keeps a single `Environment` for the whole session, so bindings persist between submissions, and a parse or runtime error is reported without ending the session. File mode creates its own environment and exits non-zero when a script fails.

## 2. Modules (Project Structure)

The project is divided into logical modules in the `src/` directory:

### `lexer.rs`
- Responsible for lexical analysis.
- Converts a character string (`String`) into a vector of tokens (`Vec<Token>`).
- Recognizes keywords (including `fun`, `and`, `or`, `not`, `execute`, `onError`, `until`, `in`, `match`) and operators (including `->`, `??`).
- Features "peek" logic for two-character operators (e.g., `==`, `+=`, `..`, `<=`, `->`, `??`).
- Automatically recognizes floats by checking if a dot following a number is not the start of a range `..`.
- Ignores `//` comments until the end of the line. Also ignores `#!` shebangs on the very first line of a file.
- Never panics on malformed input. An unknown character, a lone `!`/`?`, an out-of-range number literal, or an unterminated string literal produce a `Token::Error(String)` carrying the message; the parser surfaces it as an ordinary syntax error. This is what makes syntax errors recoverable and the REPL resilient.

### `ast.rs`
- Contains the structural definitions of the Abstract Syntax Tree (AST).
- `Expr`: Expressions that return a value when evaluated. Includes math operations, variable access, function calls, `If` (acting as a ternary expression), `ExecuteCatch` (error handling), `Unary` (negation/logical not), `Dict` (map literals), `Match` (pattern matching), `NullCoalesce` (nullish coalescing), and `Lambda` (anonymous functions).
- `Stmt`: Statements that perform actions in the code (declarations, loops, assignments) but do not return a value. Includes `LoopIn` (array iteration), `LoopBlock` (infinite loop), and `Until` (conditional break).
- Recursive trees are wrapped in `Box<T>` due to Rust's size requirements for structs.
- A function's parameter list and body (`FuncDecl`, `Lambda`) are stored behind `Rc` so that creating a callable value clones a pointer instead of deep-copying the whole body on every lookup.

### `parser.rs`
- Implements *Recursive Descent Parsing*.
- Builds the AST based on the `Vec<Token>` returned by the lexer.
- Separates expression parsing into one function per precedence level, loosest to tightest: `parse_expr` (`??`) -> `parse_or` -> `parse_and` -> `parse_equality` (`==`, `!=`) -> `parse_comparison` (`<`, `>`, `<=`, `>=`) -> `parse_additive` (`+`, `-`) -> `parse_term` (`*`, `/`, `%`) -> `parse_unary` (`-`, `not`) -> `parse_factor` (values, parentheses). Each level delegates to the next-tighter one, so `and` binds tighter than `or` and comparisons bind looser than arithmetic.
- Uses desugaring mechanisms: e.g., `+=` is turned into an assignment tree with addition on the fly, and string interpolation `"a {b}"` creates an internal Lexer and Parser instance to return an `Expr::Binary("a " + b)` expression.
- Context-aware parsing in `parse_factor`: if `{` is encountered while expecting a value, it is parsed as a `Dict` literal; if encountered as a statement, it's a code block.
- `if`, `execute`, and `match` are parsed as expressions, allowing them to be used inline (e.g., `var x = if ...`).
- Function and method calls are parsed "postfix" at the end of `parse_factor` using a single loop, allowing for unlimited chaining (e.g., `foo()().bar()[0]`).

### `interpreter.rs`
- Implements the execution logic on the AST.
- The core architecture relies on `Rc<RefCell<T>>` for shared mutability and memory efficiency:
  - `Environment` is wrapped in `Rc<RefCell<Environment>>`. This implements the Scope Chain pattern with a `parent: Option<Rc<RefCell<Environment>>>` field, allowing functions and closures to share state and read global variables without cloning the entire environment. Every block (`if`, `match`, `execute`/`onError`, and all three loop forms) runs in its own child scope, so a declaration inside a block, a loop iterator, or an `onError` variable does not leak into or overwrite the surrounding scope. Child scopes are cheap because the extension registry is resolved through the parent chain (`find_extension`) instead of being copied per scope.
  - `Value::Array` (`Rc<RefCell<Vec<Value>>>`, `Rc<Cell<bool>>`) and `Value::Dict` (`Rc<RefCell<HashMap<String, Value>>>`, `Rc<Cell<bool>>`) are containers. Each carries an **immutable flag** that travels with the object. `push` and indexed assignment check this flag, so a `let` container is read-only however it is reached.
- **Function values capture their defining scope weakly, and functions are transient.** A `Value::Function` stores a `Weak<RefCell<Environment>>` of the scope it was written in (`Rc::downgrade(env)` at `Stmt::FuncDecl`/`Expr::Lambda`), not a strong `Rc`. A weak reference never keeps the environment alive, so the `environment → function → environment` cycle that reference counting cannot collect cannot form — dropping the last external handle to an environment frees it (there is a regression test that downgrades an environment, drops it, and asserts the `Weak` no longer upgrades). `execute_function` upgrades the weak capture when the function is called; in a valid program the function is a callback still on the call stack so its scope is alive, and if the upgrade ever fails it returns a clean runtime error instead of misbehaving. To keep the weak capture sound, functions are restricted to **transient synchronous callbacks**: `Stmt::FuncDecl` is allowed only in the global scope, and any attempt to store a function value — assignment, `let`/`var` binding, `return`, an array or dictionary literal, an indexed write, or `push` — is rejected (`reject_stored_function`). A function may still be passed directly as a call argument and can see the local variables of its lexical scope while it runs.
- **Value semantics for containers.** `deep_bind(value, immutable)` gives each binding its own container: a container reached through an alias is deep-copied and every level is tagged with the binding's mutability (`let` → immutable, `var` → mutable), while a uniquely-owned value (a fresh literal, or a function's return value) is retagged in place to avoid a needless copy. It is called at `Stmt::VarDecl`, `Stmt::Let`, `Stmt::Assign`, and when storing into a container slot (indexed assignment, `push`). The result is that two named variables never share one object. **Function parameters are the deliberate exception**: `execute_function` clones the argument's `Rc` (shares by reference), so immutability travels into the callee and a mutable container can be modified in place by the function — the one intentional aliasing path.
- The `VarInfo` struct stores the `Value`, a `is_const` flag (whether the *name* can be rebound), and an optional `type_name`. This enables **Runtime Type Checking**: if `type_name` is specified, the interpreter validates values during declaration (`Stmt::VarDecl`, `Stmt::Let`) and assignment (`Stmt::Assign`). Only known type names are accepted; an unknown annotation is rejected up front (`is_known_type`). Note that a container's *mutability* is a property of the object (the immutable flag), separate from `is_const` which only governs rebinding the name.
- The interpreter core is "dumb" and agnostic to specific types. It contains no hard-coded methods (like `push` or `read`). It only understands AST evaluation, scope management, and control flow.
- Integer arithmetic uses checked operations (`checked_add`, `checked_div`, `checked_neg`, …): an overflow becomes an `InterpErr::Err` runtime error instead of panicking in debug builds or wrapping silently in release builds.
- Uses a custom error system `InterpErr` with `Return`, `Break`, `Continue`, and `Err` variants. Thanks to error propagation (`?`), control flow statements "bubble up" through nested blocks. The `Until` statement leverages this by returning `InterpErr::Break` if its condition is true.
- Introduces `eval_block_as_expr`, a helper that evaluates a block of statements (in a fresh child scope) and returns the value of the last expression, enabling `if`, `execute`, and `match` to act as expressions.
- `execute_function` is a public helper method that executes a `Value::Function` or `Value::Builtin` with given arguments. This allows the standard library (e.g., `.map()`, `.filter()`) to execute lambdas/closures passed from the language.
- `ExecuteCatch` catches `InterpErr::Err` (binding the message to a variable) while letting control flow errors (`Return`, `Break`, `Continue`) propagate naturally.

### `stdlib.rs`
- Acts as the language's standard library.
- Registers global built-in functions (`print`, `input`, `args`, `exit`, `shell`) and pure extension methods into the `Environment`.
- Implements extension methods for Strings (`upper`, `lower`, `trim`, `contains`, `replace`, `split`) and Arrays (`contains`, `join`, `push`, `map`, `filter`). Array methods like `map` and `filter` utilize the `Environment::execute_function` helper to run passed lambdas.
- Extension methods validate their argument count with `check_arity` and return a runtime error instead of panicking when called with too few arguments. `push` additionally checks the receiver's immutable flag (refusing a `let` array) and stores an independent copy of the element via `Environment::deep_bind`.
- Keeping this logic separate from `interpreter.rs` ensures the interpreter core remains lightweight and generic.

### `modules/`
- Handles external or standard library extensions loaded via the `use` keyword.
- `modules/io.rs`: Implements a `File` object using a `Dict` under the hood (storing `path` and `__type__` keys). It registers a global `file(path)` constructor and extension methods (`read`, `write`, `append`, `exists`). This design keeps the interpreter completely agnostic to file operations.

### `main.rs`
- Program entry point. Handles the CLI, reads input from the console or a file, and invokes the modules in the correct order: Lexer -> Parser -> Interpreter.
- `run_code` takes the `Environment` as a parameter and returns `Result<(), ()>` instead of exiting the process, so the REPL can reuse one environment across submissions and keep running after an error. File mode creates its own environment and maps failure to exit code `1` (success `0`).

### `tests.rs`
- Unit testing module. Uses the `#[cfg(test)]` attribute.
- Conducts integration tests of the entire pipeline: Lexer -> Parser -> Interpreter, verifying results, AST structures, and correct error throwing (including logical operators, operator precedence, loops, dictionaries, safe dictionary access, error handling, pattern matching, closures, file I/O, type checking, block scoping, integer overflow, and container value semantics / immutability). (Currently 99 passing tests).

## 3. Dependencies
The project relies entirely on the Rust standard library (`std`). It does not use any external crates.
- `std::collections::HashMap` (for environments, arrays, and dictionaries).
- `std::io` and `std::fs` (for CLI, file interactions, and I/O module).
- `std::env` (for reading startup arguments).
- `std::process` (for executing shell commands and handling exit codes).
- `std::cell::RefCell`, `std::cell::Cell`, and `std::rc::Rc` (for shared mutability, the container immutable flag, and memory management).
