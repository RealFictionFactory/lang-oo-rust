# `Ó` Programming Language

`Ó` (pronounced "OO" like in "mood") is a small, dynamically typed, interpreted programming language built entirely in Rust. It was designed with readability and a natural feel in mind, featuring a clean syntax without semicolons and optional type annotations.

## Project Goals

This is a hobbyist project created primarily for educational purposes. The main goals were:
1. To learn the **Rust programming language** (focusing on ownership, borrowing, enums, pattern matching, and error handling).
2. To understand **how programming languages work under the hood** by building a complete pipeline from scratch: a Lexer, a Parser (generating an Abstract Syntax Tree), and a Tree-Walking Interpreter.

## Features

* **Clean Syntax:** No semicolons required. Newlines determine the end of statements.
* **Optional & Enforced Typing:** Variables can be strictly typed with default values (`var x is Number`) or dynamically inferred (`var x = 10`). If a type is specified, it is strictly enforced at runtime.
* **Advanced Control Flow:** 
  * `if` and `match` can be used as statements **and** as expressions.
  * Versatile `loop` keyword: range loops, array iteration, and infinite loops with conditional `until` breaks.
  * Logical operators (`and`, `or`, `not`) with short-circuit evaluation.
* **First-Class Functions & Closures:** Assign functions to variables, pass them as arguments, and capture state with lambdas (`var f = fun(x) { return x * 2 }`).
* **Error Handling:** `execute { ... } onError(err) { ... }` blocks to catch and handle runtime errors gracefully.
* **Data Structures:** Arrays and Dictionaries (`{"key": value}`) are passed by reference, making them mutable inside functions. String interpolation (`"Hello {name}!"`) is supported natively.
* **Safe Access:** Missing keys in Dictionaries return `Null` instead of crashing. The nullish coalescing operator (`??`) provides fallback values.
* **Rich Standard Library:** Built-in extension methods for Strings (`trim`, `contains`, `replace`, `split`) and Arrays (`contains`, `join`, `push`). Supports functional programming with `.map(fun)` and `.filter(fun)`.
* **File I/O:** A built-in `File` object (loaded via `use io`) for reading, writing, and appending to files.
* **Shell Scripting Support:** Execute system commands (`shell()`), read CLI arguments (`args()`), handle exit codes (`exit()`), and use shebangs (`#!`) to run scripts natively in Unix shells.

## Quick Example

Here is a taste of what writing in "`Ó`" looks like:

```text
#!/usr/bin/env ooi

use io

// Function with recursion
fun factorial(n) {
  if n <= 1 { return 1 }
  return n * factorial(n - 1)
}

// Global input function with method chaining
var name = input("What is your name? ")
var num = input("Enter a number to calculate its factorial: ").asNumber()

print("Hello, {name}!")

// Dictionaries, 'match' expression, and Nullish Coalescing (??)
var user = {"name": name, "age": num}
var role = user["role"] ?? "guest"

var status = match user["age"] {
    0 -> "infant"
    _ -> "person"
}
print("You are a {role} {status}!")

// Functional programming with closures
var nums = [1, 2, 3, 4, 5]
var doubled = nums.map(fun(x) { return x * 2 })
print("Doubled: {doubled}")

// File I/O
var f = file("output.txt")
f.write("Factorial of {num} is {factorial(num)}")
print("File exists: {f.exists()}")
```

## Architecture

The interpreter is divided into clean, separate modules:
* `lexer.rs` - Tokenizes the raw source code.
* `parser.rs` - Converts tokens into an Abstract Syntax Tree (AST) using Recursive Descent Parsing.
* `ast.rs` - Definitions for AST nodes (Expressions and Statements).
* `interpreter.rs` - Walks the AST and executes the program, managing scopes, environments, and runtime type checking. The core is kept "dumb" and generic.
* `stdlib.rs` - The standard library (global functions and extension methods).
* `modules/` - Loadable extension modules (like `io` for file operations).

## How to Run

Make sure you have the [Rust toolchain](https://www.rust-lang.org/tools/install) installed.

1. **REPL mode (Interactive):**
   ```bash
   cargo run
   ```
   Type your code and press `.` on an empty line to execute.

2. **File mode:**
   ```bash
   cargo run -- script.oo
   ```

3. **Shell Script mode (Unix/Linux/macOS):**
   Install the binary globally: `cargo install --path .`
   Then make your script executable (`chmod +x script.oo`) and run it directly:
   ```bash
   ./script.oo
   ```

---
*Note: This project is an ongoing learning experiment and not intended for production use.*
