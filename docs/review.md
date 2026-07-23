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

2. **Function bodies were deep-copied on every call** — fixed in *perf: share function params and body via Rc*  
   Files: `src/ast.rs`, `src/parser.rs`, `src/interpreter.rs`  
   `Value::Function` owned `Vec<String>` parameters and a `Vec<Stmt>` body by value, and `Environment::get()` clones the whole `VarInfo` on every variable lookup. Calling a function therefore deep-copied its entire AST first, making call cost proportional to the function's source size rather than to the work it does. Parameters and body now sit behind `Rc` in `Expr::Lambda`, `Stmt::FuncDecl` and `Value::Function`, so a lookup clones three pointers.

   Measured with 200,000 calls, best of three, release build. The padded case adds 400 statements inside an `if false` block, so they are never executed — the cost was purely the copying:

   | Case | Before | After |
   | --- | --- | --- |
   | Empty body | 0.15s | 0.11s |
   | 400 never-executed statements in body | 1.93s | 0.12s |
   | Lambda allocated inside a hot loop | 0.20s | 0.16s |

   Call cost is now flat in body size instead of growing 13x.

3. **Operator precedence was wrong for comparisons and for `and`/`or`** — fixed in *fix: give each operator its own precedence level*  
   File: `src/parser.rs`  
   `parse_logic()` handled `+`, `-` and every comparison on a single left-associative level, and `parse_expr()` did the same for `and` and `or`. Two groupings were therefore wrong:

   | Expression | Before | After |
   | --- | --- | --- |
   | `1 < 2 + 3` | `(1 < 2) + 3` → runtime error | `1 < (2 + 3)` → `true` |
   | `true or false and false` | `(true or false) and false` → `false` | `true or (false and false)` → `true` |

   The flat level is replaced by one function per precedence tier — `parse_or`, `parse_and`, `parse_equality`, `parse_comparison`, `parse_additive` — each delegating to the next-tighter one. `??` keeps its existing position as the loosest operator, and unary `-`/`not` keep theirs. Regression tests cover both groupings.

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

7. **`let` protection is bypassed for mutable containers**  
   Files: `src/interpreter.rs`, `src/stdlib.rs`  
   Indexed assignment mutates arrays/dictionaries without checking `is_const`. Mutating extension methods also bypass it: `let xs = [1]; xs.push(2)` succeeds. Because values are shared through `Rc<RefCell<_>>`, an alias can mutate a container held by a `let` binding as well.

8. **`onError` variable overwrites and leaks into surrounding scope**  
   File: `src/interpreter.rs`  
   `ExecuteCatch` inserts its error variable into the current environment. It can overwrite an existing `let` binding and remains visible after the handler completes, rather than being handler-local.

9. **Loop iterator variables overwrite surrounding bindings**  
   File: `src/interpreter.rs`  
   Range and array loops insert their iterator into the current environment. `var i = 7; loop i from 1..2 {}; print(i)` prints `1`; the loop variable is neither scoped nor restored.

10. **Unknown declared types are accepted when initialized**  
    File: `src/interpreter.rs`  
    `value_matches_type()` returns `true` for every unknown type name. `var x is MadeUp = 1` succeeds, while `var x is MadeUp` fails because no default value exists. This makes type handling inconsistent.

11. **Closure/environment reference cycles leak memory**  
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

- `cargo test --all-targets` passed: 72 passed, 0 failed (68 at the time of review, plus 4 regression tests added with the fixes above).
- Targeted runtime checks reproduced REPL state loss, `let` mutation, unchecked-argument panic, literal-overflow panic, arithmetic-overflow panic, unterminated-string acceptance, error-variable leakage, loop-variable overwrite, and unknown-type acceptance.
- `cargo clippy --all-targets -- -D warnings` currently fails with 35 diagnostics. Most are style/idiom diagnostics; they are not counted as functional findings above.
