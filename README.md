# Ó Programming Language

Ó (pronounced "OO" like in "mood") is a small, dynamically typed, interpreted programming language built entirely in Rust. It was designed with readability and a natural feel in mind, featuring a clean syntax without semicolons and optional type annotations.

## Project Goals

This is a hobbyist project created primarily for educational purposes. The main goals were:
1. To learn the **Rust programming language** (focusing on ownership, borrowing, enums, pattern matching, and error handling).
2. To understand **how programming languages work under the hood** by building a complete pipeline from scratch: a Lexer, a Parser (generating an Abstract Syntax Tree), and a Tree-Walking Interpreter.

## Features

* **Clean Syntax:** No semicolons required. Newlines determine the end of statements.
* **Optional Typing:** Variables can be strictly typed with default values (`var x is Number`) or dynamically inferred (`var x = 10`).
* **Type Promotion:** Automatic promotion from `Number` (i64) to `Decimal` (f64) in mathematical operations.
* **Advanced Control Flow:** 
  * `if` can be used as a statement **and** as an expression (e.g., `var x = if cond { 1 } else { 2 }`).
  * Versatile `loop` keyword: range loops (`loop i from 1..10`), array iteration (`loop x in arr`), and infinite loops (`loop { ... }`) with conditional `until (cond)` breaks.
  * Logical operators (`and`, `or`, `not`) with short-circuit evaluation.
* **Error Handling:** `execute { ... } onError(err) { ... }` blocks to catch and handle runtime errors gracefully.
* **Data Structures:** Arrays, Dictionaries (`{"key": value}`), and String interpolation (`"Hello {name}!"`).
* **Lightweight Standard Library:** Global functions like `print` and `input`, alongside a clean extension method system (e.g., `"text".upper()`, `input("Age: ").asNumber()`).

## Quick Example

Here is a taste of what writing in "Ó" looks like:

```text
// Function with recursion
func factorial(n) {
  if n <= 1 {
    return 1
  }
  return n * factorial(n - 1)
}

// Global input function with method chaining
var name = input("What is your name? ")
var num = input("Enter a number to calculate its factorial: ").asNumber()

print("Hello, {name}!")

// Dictionaries and 'if' as an expression
var user = {"name": name, "age": num}
var status = if user["age"] >= 18 { "adult" } else { "minor" }
print("Status: {status}")

// Error handling
var result = execute {
  print("Factorial of {num} is {factorial(num)}")
} onError(err) {
  print("Something went wrong: {err}")
}
```

## Architecture

The interpreter is divided into clean, separate modules:
* `lexer.rs` - Tokenizes the raw source code.
* `parser.rs` - Converts tokens into an Abstract Syntax Tree (AST) using Recursive Descent Parsing.
* `ast.rs` - Definitions for AST nodes (Expressions and Statements).
* `interpreter.rs` - Walks the AST and executes the program, managing scopes and environments.
* `stdlib.rs` - The standard library (global functions and extension methods).
* `modules/` - Placeholder for future loadable extension modules.

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

---
*Note: This project is an ongoing learning experiment and not intended for production use.*
```
