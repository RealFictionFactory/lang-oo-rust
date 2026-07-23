# Codebase Review Findings

Date: 2026-07-23

## Fixed

1. **Extension methods ran while the environment was borrowed** — fixed in *fix: release env borrow before running extension methods*  
   File: `src/interpreter.rs`  
   `Expr::MethodCall` looked up the extension through `env.borrow()` and held that `Ref` across the call itself. Extensions that run user code (`map()`, `filter()`) therefore executed the callback while the environment was immutably borrowed, so assigning to a variable in an enclosing scope hit `borrow_mut()` and panicked with `RefCell already borrowed`:

   ```
   var total = 0
   [1, 2, 3].map(fun(x) { total = total + x
   return x })
   ```

   The panic was a process abort rather than an `InterpErr`, so `execute`/`onError` could not catch it. The fix copies the function pointer out with `.copied()` before the call; regression tests cover `map()` and `filter()` callbacks assigning to an enclosing scope.

## Confirmed Issues

1. **REPL exits on first syntax or runtime error**  
   File: `src/main.rs`  
   `run_code()` calls `std::process::exit(1)` on parse/runtime errors. One invalid REPL submission terminates the process instead of returning to the prompt.

2. **REPL does not preserve state between submissions**  
   File: `src/main.rs`  
   `run_code()` creates a new `Environment` for every submission. For example, after submitting `var x = 1`, a later `print(x)` fails because `x` is no longer defined.

3. **Unchecked stdlib argument indexing panics**  
   File: `src/stdlib.rs`  
   Several extension methods directly access `args[0]` or `args[1]` without validating argument count. For example, `[].push()` panics instead of producing an interpreter error.

4. **Lexer panics rather than reporting recoverable syntax errors**  
   File: `src/lexer.rs`  
   Unknown characters use `panic!`; numeric parsing uses `unwrap()`. An integer literal larger than `i64::MAX` aborts the process with a Rust panic.

5. **Unterminated strings are accepted silently**  
   File: `src/lexer.rs`  
   The string lexer stops at EOF without checking for a closing quote. `var x = "unterminated` completes successfully instead of reporting a syntax error.

6. **Integer arithmetic can panic on overflow**  
   File: `src/interpreter.rs`  
   Arithmetic and unary negation use unchecked `i64` operations. In debug builds, `9223372036854775807 + 1` panics instead of returning a language-level runtime error.

7. **Operator precedence is incorrect for comparisons and arithmetic**  
   File: `src/parser.rs`  
   `parse_logic()` gives `+/-` and comparison operators the same precedence. `1 < 2 + 3` is parsed as `(1 < 2) + 3`, which then fails at runtime.

8. **`let` protection is bypassed for mutable containers**  
   Files: `src/interpreter.rs`, `src/stdlib.rs`  
   Indexed assignment mutates arrays/dictionaries without checking `is_const`. Mutating extension methods also bypass it: `let xs = [1]; xs.push(2)` succeeds. Because values are shared through `Rc<RefCell<_>>`, an alias can mutate a container held by a `let` binding as well.

9. **`onError` variable overwrites and leaks into surrounding scope**  
   File: `src/interpreter.rs`  
   `ExecuteCatch` inserts its error variable into the current environment. It can overwrite an existing `let` binding and remains visible after the handler completes, rather than being handler-local.

10. **Loop iterator variables overwrite surrounding bindings**  
    File: `src/interpreter.rs`  
    Range and array loops insert their iterator into the current environment. `var i = 7; loop i from 1..2 {}; print(i)` prints `1`; the loop variable is neither scoped nor restored.

11. **Unknown declared types are accepted when initialized**  
    File: `src/interpreter.rs`  
    `value_matches_type()` returns `true` for every unknown type name. `var x is MadeUp = 1` succeeds, while `var x is MadeUp` fails because no default value exists. This makes type handling inconsistent.

12. **Closure/environment reference cycles leak memory**  
    File: `src/interpreter.rs`  
    A function stores a strong `Rc` reference to its defining environment, and that environment stores the function. Function declarations and stored lambdas can therefore form `Rc` cycles that are never released.

## Performance Concerns

1. **Full-array cloning in iteration helpers**  
   Files: `src/interpreter.rs`, `src/stdlib.rs`  
   `loop in`, `.map()`, and `.filter()` clone complete arrays before iteration. This adds O(n) time and memory overhead per operation, although it currently avoids `RefCell` borrow conflicts when callbacks mutate the array.

2. **Extension registry cloned for every function call**  
   File: `src/interpreter.rs`  
   `with_parent()` clones the full `extensions` map for each function-local environment. Function-call cost grows with registered extension count.

## Verification

- `cargo test --all-targets` passed: 70 passed, 0 failed (68 at the time of review, plus 2 regression tests added with the fix above).
- Targeted runtime checks reproduced the precedence bug, REPL state loss, `let` mutation, unchecked-argument panic, literal-overflow panic, arithmetic-overflow panic, unterminated-string acceptance, error-variable leakage, loop-variable overwrite, and unknown-type acceptance.
- `cargo clippy --all-targets -- -D warnings` currently fails with 35 diagnostics. Most are style/idiom diagnostics; they are not counted as functional findings above.
