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
* **Core Constructs:** `if/else if/else`, `loop` (with `break` and `continue`), and user-defined `func`tions with recursion support.
* **Data Structures:** Arrays and String interpolation (`"Hello {name}!"`).
* **Extension Methods:** A modular standard library system. Instead of global functions, capabilities like `input` are loaded via `use io` and called directly on objects (e.g., `0.input("Prompt: ")`).

## Quick Example

Here is a taste of what writing in "Ó" looks like:

```text
use io

// Function with recursion
func factorial(n) {
  if n <= 1 {
    return 1
  }
  return n * factorial(n - 1)
}

// Type-annotated variable
var name is String
name = "".input("What is your name? ")

// Type-inferred variable with extension method
var num = 0.input("Enter a number to calculate its factorial: ")

print("Hello, {name}!")
print("The factorial of {num} is {factorial(num)}")
```

## Architecture

The interpreter is divided into clean, separate modules:
* `lexer.rs` - Tokenizes the raw source code.
* `parser.rs` - Converts tokens into an Abstract Syntax Tree (AST) using Recursive Descent Parsing.
* `ast.rs` - Definitions for AST nodes (Expressions and Statements).
* `interpreter.rs` - Walks the AST and executes the program, managing scopes and environments.
* `stdlib.rs` & `modules/` - The standard library and loadable extension modules (like `io`).

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