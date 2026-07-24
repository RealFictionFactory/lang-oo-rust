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

4. **Blocks did not introduce a scope, and the extension registry was copied per scope** — fixed in *fix: give blocks their own scope*  
   File: `src/interpreter.rs`  
   Only function calls created an environment. `if`, `match`, `execute`/`onError` and all three loop forms ran their bodies directly in the enclosing scope, so three things went wrong at once:

   | Program | Before | After |
   | --- | --- | --- |
   | `var i = 7` then `loop i from 1..2 {}` | `i` is `1` | `i` is `7` |
   | `let e = 1` then `execute {…} onError(e) {…}` | `e` holds the error message afterwards | `e` is `1`, handler-local |
   | `if true { var x = 5 }` then `x` | `5` | `Variable 'x' is not defined` |

   Blocks now run in a child scope: `eval_block_as_expr` creates one, loops own a scope holding their iterator, and `onError` seeds its error variable into the handler's own scope via the new `eval_block_in_scope`. Assignment still walks the parent chain, so blocks read and mutate enclosing bindings as before.

   This was only affordable because `with_parent()` no longer clones the whole `extensions` map for each scope — the previously separate performance concern. Extensions resolve through the parent chain instead, via `find_extension`. Creating a scope became cheap enough that adding scopes to every block cost nothing measurable, and function calls got faster (200,000 calls, best of three, release):

   | Case | Before | After |
   | --- | --- | --- |
   | Function calls | 0.12s | 0.06s |
   | `if` blocks, one new scope each | 0.04s | 0.04s |
   | Lambda allocated inside a hot loop | 0.16s | 0.07s |

   Still open, and listed below: loop bodies share one scope across all iterations rather than getting a fresh one per iteration, so closures created in a loop all observe the final value of the iterator.

5. **The REPL exited on the first error and kept no state between submissions** — fixed in *fix: make the REPL persistent and error-tolerant*  
   File: `src/main.rs`  
   `run_code()` called `std::process::exit(1)` on any parse or runtime error and built a fresh `Environment` for every submission, so one bad line ended the session and `var x = 1` was forgotten by the next `print(x)`.

   | REPL session | Before | After |
   | --- | --- | --- |
   | `var x = 1` then `print(x)` | `Variable 'x' is not defined` | `1` |
   | `print(undefined_var)` then `var z = 5` then `print(z + 1)` | process exits at the error | error printed, then `6` |

   `run_code()` now takes the environment as a parameter and returns `Result<(), ()>` instead of exiting. `main` creates one environment for the whole REPL session and reuses it; errors are printed and the prompt returns. File mode still creates its own environment and exits non-zero when a script fails, which is the right behaviour for running a script.

   Regression tests cover state persisting across submissions and a session surviving both a parse error and a runtime error. Lexer `panic!`s (fixed next) previously aborted the process before `run_code` could report them; with those gone the REPL now survives every input.

6. **The lexer panicked instead of reporting recoverable syntax errors** — fixed in *fix: make the lexer emit error tokens instead of panicking*  
   Files: `src/lexer.rs`, `src/parser.rs`  
   `tokenize()` used `panic!` for an unknown character and a lone `!` or `?`, and `unwrap()` when parsing a number literal, so a stray `@` or an integer larger than `i64::MAX` aborted the whole process — uncatchable, and fatal to the REPL.

   | Input | Before | After |
   | --- | --- | --- |
   | `var x = @` | `panic: Unknown character: @` | `Syntax error: Unknown character: '@'` |
   | `print(99999999999999999999)` | `panic: called unwrap() on Err(PosOverflow)` | `Syntax error: Number literal '…' is out of range for Number (i64)` |

   The lexer now emits a `Token::Error(String)` carrying the message and keeps going; the parser surfaces that message as a normal parse error, so file mode exits non-zero and the REPL prints it and continues. `tokenize()` keeps returning `Vec<Token>`, so its many existing callers are unchanged.

   One rough edge remains: a stray operator fragment such as `!` inside a call's argument list is reported through the generic "Expected ',' or ')'" path rather than the lexer's specific message, because the argument-separator check consumes the error token first. It is still a reported error, not a panic.

7. **Unterminated string literals were accepted silently** — fixed in *fix: report unterminated string literals*  
   File: `src/lexer.rs`  
   The string lexer consumed characters until either a closing quote or end of input, and could not tell the two apart: `var x = "oops` reached EOF and produced a `String` token holding everything after the quote, so a missing quote passed as valid code. The loop now records whether it saw a closing quote and emits a `Token::Error` when it did not, which the parser surfaces like any other syntax error. Properly closed strings — including the empty string, multi-line strings, escapes and interpolation — are unchanged.

## Confirmed Issues

1. **Unchecked stdlib argument indexing panics**  
   File: `src/stdlib.rs`  
   Several extension methods directly access `args[0]` or `args[1]` without validating argument count. For example, `[].push()` panics instead of producing an interpreter error.

2. **Integer arithmetic can panic on overflow**  
   File: `src/interpreter.rs`  
   Arithmetic and unary negation use unchecked `i64` operations. In debug builds, `9223372036854775807 + 1` panics instead of returning a language-level runtime error.

3. **`let` protection is bypassed for mutable containers**  
   Files: `src/interpreter.rs`, `src/stdlib.rs`  
   Indexed assignment mutates arrays/dictionaries without checking `is_const`. Mutating extension methods also bypass it: `let xs = [1]; xs.push(2)` succeeds. Because values are shared through `Rc<RefCell<_>>`, an alias can mutate a container held by a `let` binding as well.

4. **Unknown declared types are accepted when initialized**  
   File: `src/interpreter.rs`  
   `value_matches_type()` returns `true` for every unknown type name. `var x is MadeUp = 1` succeeds, while `var x is MadeUp` fails because no default value exists. This makes type handling inconsistent.

5. **Closure/environment reference cycles leak memory**  
   File: `src/interpreter.rs`  
   A function stores a strong `Rc` reference to its defining environment, and that environment stores the function. Function declarations and stored lambdas can therefore form `Rc` cycles that are never released.

6. **Closures created in a loop all capture the final iterator value**  
   File: `src/interpreter.rs`  
   A loop owns one scope shared by every iteration rather than creating a fresh one per iteration, and closures capture that scope by reference. `var fs = []; loop i from 0..3 { fs.push(fun() { return i }) }` leaves every closure returning `2`.

## Performance Concerns

1. **Full-array cloning in iteration helpers**  
   Files: `src/interpreter.rs`, `src/stdlib.rs`  
   `loop in`, `.map()`, and `.filter()` clone complete arrays before iteration. This adds O(n) time and memory overhead per operation, although it currently avoids `RefCell` borrow conflicts when callbacks mutate the array.

## Verification

- `cargo test --all-targets` passed: 84 passed, 0 failed (68 at the time of review, plus 16 regression tests added with the fixes above).
- Targeted runtime checks reproduced `let` mutation, unchecked-argument panic, arithmetic-overflow panic, unknown-type acceptance, and loop-closure capture.
- `cargo clippy --all-targets -- -D warnings` currently fails with 35 diagnostics. Most are style/idiom diagnostics; they are not counted as functional findings above.
