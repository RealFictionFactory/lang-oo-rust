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

8. **Extension methods panicked when called with too few arguments** — fixed in *fix: check argument count in stdlib extension methods*  
   File: `src/stdlib.rs`  
   Seven extension methods (`push`, `contains`, `replace`, `split`, `join`, `map`, `filter`) indexed `args[0]`/`args[1]` without checking the argument count, so `[].push()` or `"x".replace("a")` aborted the process with an out-of-bounds panic. A new `check_arity(method, args, expected)` helper is called at the top of each; a short call now returns e.g. `push() expects 1 argument(s), got 0` as a normal interpreter error. `io.rs` already used `args.get()` and was unaffected.

9. **Integer arithmetic overflowed unchecked** — fixed in *fix: return a runtime error on integer overflow*  
   File: `src/interpreter.rs`  
   `+`, `-`, `*`, `/`, `%` on `Number` and unary negation used raw `i64` operators, so `9223372036854775807 + 1` panicked in debug builds and silently wrapped to a negative number in release builds — a wrong answer either way. `i64::MIN / -1` and negating `i64::MIN` overflow too, past the existing divide-by-zero guard.

   | Expression | Before (release) | After |
   | --- | --- | --- |
   | `9223372036854775807 + 1` | `-9223372036854775808` | `Runtime error: integer overflow in 9223372036854775807 + 1` |
   | `i64::MIN / -1` | panic | reported overflow error |

   Each operator now uses its `checked_*` form and returns a runtime error when it overflows, so the result is either correct or a catchable error, never a wrap or a panic. Ordinary arithmetic is unchanged. (The message carries its own `Runtime error:` prefix to match the adjacent divide-by-zero messages, which double up with the caller's prefix — a pre-existing cosmetic wart left as is.)

10. **`let` protection was bypassed for mutable containers** — fixed in *fix: enforce let on indexed writes and mutating methods*  
    File: `src/interpreter.rs`  
    A `let`-bound array or dictionary could still be changed in place: `let xs = [1]; xs[0] = 9` and `let xs = [1]; xs.push(2)` both succeeded, because indexed assignment never checked `is_const` and a mutating method received the container's `Rc<RefCell>` with no knowledge of the binding.

    | Program | Before | After |
    | --- | --- | --- |
    | `let xs = [1,2]; xs[0] = 9` | mutates to `[9, 2]` | `Cannot modify 'xs': it is declared with 'let'` |
    | `let xs = [1]; xs.push(2)` | mutates to `[1, 2]` | `Cannot call 'push' on 'xs': it is declared with 'let'` |

    `IndexAssign` now rejects a write when the target binding is const. `MethodCall` refuses an in-place mutating method (`is_mutating_method`, currently just `push`) on a const receiver, while pure methods like `map`, `filter`, `length` and `upper` remain allowed. `var` containers are unaffected.

    One case is intentionally left open, and listed below: because containers are shared by `Rc<RefCell>`, a container reached through a separate `var` alias (`let xs = [1]; var ys = xs; ys.push(2)`) is still mutable. Closing it would require either value-copy semantics on assignment or freezing the container itself, both of which conflict with the language's deliberate pass-by-reference behaviour.

11. **Unknown declared types were accepted when initialized** — fixed in *fix: reject unknown type annotations at declaration*  
    File: `src/interpreter.rs`  
    `value_matches_type()` returned `true` for any unrecognised type name, so `var x is MadeUp = 1` was accepted while `var x is MadeUp` (no initializer) failed in `get_default_value` — the annotation was validated in one path but not the other.

    | Declaration | Before | After |
    | --- | --- | --- |
    | `var x is MadeUp = 1` | accepted | `Unknown type: MadeUp` |
    | `var x is MadeUp` | `Unknown type: MadeUp` | `Unknown type: MadeUp` (unchanged) |

    A new `is_known_type` lists the recognised types (`Number`, `Decimal`, `String`, `Bool`, `Array`, `Dict`, `Null`); both `var` and `let` reject an unknown annotation before evaluating the initializer, so the two paths agree. `value_matches_type()`'s fallthrough is now `false` for defence, and a genuine mismatch (`var x is Number = "s"`) still reports `Type mismatch` rather than `Unknown type`.

12. **Container mutability is now a property of the object; assignment copies** — fixed in *fix: value semantics for containers with object-level immutability*  
    Files: `src/interpreter.rs`, `src/stdlib.rs`, `src/modules/io.rs`  
    Previously `let` protected only the *name*, so a `var` alias bound to the same `Rc<RefCell>` could mutate a `let` container: `let xs = [1]; var ys = xs; ys.push(2)` changed `xs`. This replaced the binding-level `let` checks (which only ever covered direct writes) with a whole-model change agreed with the language owner:

    - **Immutability lives on the object.** `Array`/`Dict` each carry an `immutable` flag (`Rc<Cell<bool>>`) that travels with the value. `push` and indexed assignment check that flag, so a `let` container is read-only however it is reached — directly, through an alias, through a function parameter, or nested inside another container.
    - **`=` gives the name its own container (value semantics).** `deep_bind` deep-copies a container that is reached through an alias and tags every level with the binding's mutability (`let` → immutable, `var` → mutable). A uniquely-owned value (a fresh literal, or a value returned from a function) is retagged in place instead of copied, so there is no needless allocation. Two named variables can never share one object.
    - **Function parameters are the one exception — they share by reference.** Immutability travels with the object, so a `let` array passed to a function is read-only inside it, while a `var` array is still mutable in place (the intentional ad-table pattern). This is the only place two names touch one object.
    - **`.toMutable()`/`.toImmutable()` were considered and dropped:** the `let`/`var` keyword already is the explicit mutable/immutable choice, so `var b = a` yields a mutable copy and `let b = a` an immutable one.

    | Program | Before | After |
    | --- | --- | --- |
    | `let xs=[1]; var ys=xs; ys.push(99)` | `xs` becomes `[1, 99]` | `xs` stays `[1]`, `ys` is `[1, 99]` |
    | `var a=[1]; var b=a; b.push(9)` | `a` becomes `[1, 9]` | `a` stays `[1]`, `b` is `[1, 9]` |
    | `fun f(a){a.push(7)}; let xs=[1]; f(xs)` | mutates `xs` | error — immutability travels |
    | `fun f(a){a.push(7)}; var xs=[1]; f(xs)` | `xs` becomes `[1, 7]` | `xs` becomes `[1, 7]` (unchanged) |
    | `let m={"a":[1]}; m["a"].push(2)` | mutates | error — deep |

    A side benefit: because indexed assignment now evaluates its target as a value, nested writes like `m[0][1] = 99` work. Scalars (Number/String/Bool) are unchanged value types; the flag applies only to Array/Dict.

13. **Closures created in a loop all captured the final iterator value** — fixed in *fix: give each loop iteration its own scope*  
    File: `src/interpreter.rs`  
    All three loop forms reused one scope across every iteration and inserted the iterator into it, so a closure created in the body captured that shared scope by reference and observed only the last value.

    | Program | Before | After |
    | --- | --- | --- |
    | `var fs=[]; loop i from 0..3 { fs.push(fun(){return i}) }` then `fs[0]() fs[1]() fs[2]()` | `2 2 2` | `0 1 2` |

    Each iteration now runs in its own child scope holding that iteration's iterator value; a closure captures the per-iteration scope. Mutating an enclosing variable from the loop body still works because assignment walks the parent chain. Per-iteration scopes are cheap (a 1,000,000-iteration loop runs in ~0.17s).

14. **Iteration helpers duplicated the whole array before iterating** — fixed in *perf: iterate arrays by index instead of cloning the whole array*  
    Files: `src/interpreter.rs`, `src/stdlib.rs`  
    `loop in`, `.map()` and `.filter()` each did `arr.borrow().clone()` — a full duplicate of the array — before iterating, in order not to hold a `RefCell` borrow across user code. They now iterate by index, cloning one element at a time under a brief borrow that is released before the callback/body runs. The array length is snapshotted, so the observable behaviour is unchanged: a callback may still read the array being iterated without a borrow panic, and elements appended by the body are not visited.

    This removes the redundant O(n) duplicate (peak memory drops by one full copy of the array) and the extra copy pass. Wall-clock time is dominated by the per-element callback/loop-body work and by the unavoidable per-element clone handed to the callback, so it is essentially unchanged — this is a memory/allocation fix, not a speed-up. The `RefCell` borrow safety that the original clone provided is preserved because no borrow is held while user code runs.

## Confirmed Issues

1. **Closure/environment reference cycles leak memory**  
   File: `src/interpreter.rs`  
   A function stores a strong `Rc` reference to its defining environment, and that environment stores the function. Function declarations and stored lambdas can therefore form `Rc` cycles that are never released.

## Performance Concerns

*(none open)*

## Verification

- `cargo test --all-targets` passed: 104 passed, 0 failed (68 at the time of review, plus 36 regression tests added with the fixes above).
- `cargo clippy --all-targets -- -D warnings` currently fails with 35 diagnostics. Most are style/idiom diagnostics; they are not counted as functional findings above.
