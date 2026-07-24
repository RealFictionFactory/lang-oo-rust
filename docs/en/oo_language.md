# Ó Language - Syntax Documentation

"Ó" is a simple, dynamically typed programming language with a scripting nature. It was designed with readability in mind, avoiding boilerplate (like semicolons) and sounding natural.

## 1. Basics
*   **No Semicolons:** End of line means end of statement. Blank lines are ignored.
*   **Comments:** Start with `//` and continue to the end of the line.
*   **Shebang:** Unix shebangs (`#!/usr/bin/env ooi`) are allowed on the first line and are ignored by the lexer.
*   **Code Blocks:** Enclosed in braces `{ ... }`. The opening brace can be on a new line.

## 2. Data Types
The language has built-in types that can be optionally specified during declaration:
*   `Number` - 64-bit integer.
*   `Decimal` - 64-bit floating-point number.
*   `String` - Text enclosed in double quotes.
*   `Bool` - Logical value `true` or `false`.
*   `Array` - Array of elements. A container whose mutability is decided by how it is declared (see *Mutability and Copying* below).
*   `Dict` - A collection of key-value pairs where keys are Strings. Created using braces `{"key": value}`. A container, like `Array`.
*   `Null` - Absence of a value (returned e.g., by functions without a `return` statement or missing dictionary keys).

`Number`, `Decimal`, `String` and `Bool` are simple value types: assigning one to another variable always copies it. `Array` and `Dict` are containers and follow the model described in *Mutability and Copying*.

## 3. Variables and Constants
Declarations use the `var` (mutable) and `let` (immutable) keywords. You can specify the type using `is Type`, which assigns a default value (`0` for numbers, `false` for Bool, `""` for String, `[]` for Array, `{}` for Dict).

```text
var x = 10
let pi = 3.14
var name is String  // defaults to ""
var arr is Array // defaults to []
```

### Runtime Type Checking
If a type is explicitly specified, the language enforces it at runtime. Attempting to assign a value of the wrong type to a typed variable will result in a runtime error. Only the built-in type names are accepted; an unknown type is rejected at declaration, whether or not an initial value is given.

```text
var age is Number = 20
age = "twenty" // Runtime error: Type mismatch: cannot assign String to variable of type Number

var x is MadeUp = 1 // Runtime error: Unknown type: MadeUp
```

### Mutability and Copying (Value Semantics)
Mutability is a property of the container itself, chosen by the keyword you declare it with:

*   `var` produces a **mutable** container — you can `push` to it and assign to its elements.
*   `let` produces an **immutable** container — any attempt to change it (or anything nested inside it) is a runtime error.

Assignment gives each name its **own** container. Assigning one container to another variable makes an independent copy, so two variables never share the same object by accident. The keyword on the left decides the copy's mutability:

```text
let xs = [1]
var ys = xs      // ys is an independent, MUTABLE copy of xs
ys.push(99)
print(xs)        // [1]      - the original is untouched
print(ys)        // [1, 99]

var a = [1]
var b = a        // independent copy again
b.push(9)
print(a)         // [1]
```

To get a mutable copy of an immutable container, simply assign it to a `var`; to freeze a snapshot of a mutable one, assign it to a `let`. There is no separate copy method — the keyword is the choice.

Mutating a `let` container is always an error, no matter how it is reached — directly, through another variable, or nested inside another container (immutability is **deep**):

```text
let xs = [1]
xs.push(2)              // Runtime error: cannot push to an immutable array

let grid = [[1], [2]]
grid[0].push(9)        // Runtime error: the nested array is immutable too
```

**Function parameters are the one exception: they are shared by reference, not copied.** This is the intended way to pass a large container into a function cheaply. Immutability travels with the object: a `let` container is read-only inside the callee, while a `var` container can be modified in place and the caller sees the change.

```text
fun fill(target) { target.push(7) }

var xs = [1]
fill(xs)
print(xs)        // [1, 7]  - a mutable container is modified in place

let ys = [1]
fill(ys)         // Runtime error: the immutable container cannot be modified inside the function
```

A container returned from a function is bound fresh at the call site, so the caller's keyword decides its mutability: `var out = f()` gives a mutable result, `let out = f()` an immutable one, regardless of how `f` built it.

## 4. Operators
*   **Arithmetic:** `+`, `-`, `*`, `/`, `%` (modulo).
*   **Unary:** `-` (negation, e.g., `-5`), `not` (logical negation, e.g., `not true`).
*   **Logical:** `and`, `or` (support short-circuit evaluation).
*   **Comparison:** `==`, `!=`, `>`, `<`, `>=`, `<=`.
*   **Assignment:** `=`, `+=`, `-=`.
*   **Nullish Coalescing:** `??` (returns the left value if it is not `Null`, otherwise evaluates and returns the right value).

*Precedence* (loosest to tightest): `??` → `or` → `and` → equality (`==`, `!=`) → comparison (`<`, `>`, `<=`, `>=`) → `+` `-` → `*` `/` `%` → unary (`-`, `not`) → values and parentheses. So `1 < 2 + 3` means `1 < (2 + 3)`, and `true or false and false` means `true or (false and false)`.

*Concatenation:* The `+` operator concatenates strings. If you concatenate a String with a Number/Decimal, the number is automatically converted to text.

*Integer overflow:* Integer (`Number`) arithmetic that would exceed the 64-bit range raises a runtime error rather than silently wrapping around.

## 5. Conditional Statements (`if` / `else`)
`if` can be used as a standard statement or as an expression that returns a value.

```text
var x = 5
if x > 10 {
  print("A lot")
} else if x == 5 {
  print("Five")
} else {
  print("A little")
}

// if as an expression
var status = if x > 10 { "big" } else { "small" }
```
*Truthiness:* In conditions, the values `0`, `0.0`, `""` (empty string), and `false` are treated as false. Everything else evaluates to true.

## 6. Pattern Matching (`match`)
`match` is a powerful alternative to long `if/else if` chains. It can be used as a statement or as an expression that returns a value. It does not fall through (no `break` needed).

```text
var x = 2

// As an expression
var name = match x {
    0 -> "zero"
    1 -> "one"
    _ -> "many" // _ is a wildcard that matches everything
}

// With block bodies
match x {
    0 -> print("zero")
    _ -> {
        var y = x * 10
        print("many: ", y)
    }
}
```

## 7. Loops (`loop`)
The `loop` keyword is highly versatile and supports multiple variants:

### Range Loop
Iterates over integer values (`Number`). Uses the `from` keyword and the `..` range operator.
```text
loop i from 1..5 {
  if i == 3 { continue } // skips 3
  print(i)
}
```

### Array Iteration Loop
Iterates over elements of an array. Uses the `in` keyword.
```text
var arr = [10, 20, 30]
loop element in arr {
  print(element)
}
```

### Infinite / Conditional Loop (`loop {}` and `until`)
A simple `loop {}` creates an infinite loop. It can be terminated using `break`, or conditionally terminated using `until (condition)`.
`until` acts as a conditional `break`. If placed at the top of the block, it acts like `while not`. If placed at the bottom, it acts like `do-while not`.

```text
// Acts like a do-while loop
var pass = ""
loop {
  pass = input("Enter password: ")
  until (pass == "secret") // Checks at the bottom
}
```

## 8. Functions
Named functions are defined with the `fun` keyword **at the top level** of a program. They return a value using `return`, support recursion, and have their own local scope.

```text
fun add(a, b) {
  return a + b
}

fun factorial(n) {
  if n <= 1 { return 1 }
  return n * factorial(n - 1)   // recursion
}

fun triple(x) { return x * 3 }

// A named function can be passed to a higher-order method by name
print([1, 2, 3].map(triple))    // [3, 6, 9]
```

### Lambdas are immediate callbacks
A lambda is an anonymous function written inline: `fun(params) { body }`. Lambdas exist to be **passed directly as callback arguments** — to `map`, `filter`, or to your own functions — and are invoked immediately during that call.

```text
var nums = [1, 2, 3]
print(nums.map(fun(x) { return x * 2 }))          // [2, 4, 6]
print(nums.filter(fun(x) { return x % 2 == 1 }))  // [1, 3]

fun apply(f, x) { return f(x) }                    // a higher-order function
print(apply(fun(y) { return y + 10 }, 5))          // 15
```

A callback **can see the variables of the scope it is written in** while it runs — locals and globals alike — because that scope is still active during the call:

```text
fun scale(values, factor) {
    return values.map(fun(x) { return x * factor })  // sees the local `factor`
}
print(scale([1, 2, 3], 10)) // [10, 20, 30]
```

### Functions are not stored
To keep the language free of memory-leaking reference cycles, a function value may only be **passed as a callback argument**. It cannot be stored: assigning one to a variable, returning it, putting it in an array or dictionary, or `push`-ing it are all errors. Named functions may only be declared at the top level (not nested inside another function).

```text
var f = fun(x) { return x }      // Error: a function cannot be assigned to a variable
fun outer() {
    return fun() { return 1 }    // Error: a function cannot be returned
}
fun outer2() {
    fun helper() { return 1 }    // Error: functions can only be declared at the top level
}
```

Consequently, closures that outlive their scope (such as a counter that remembers its own `count` and is returned for later use) are not supported. Pass state in as an argument instead.

## 9. Arrays
Created using square brackets `[]`. Indexed from `0`. A `var` array is mutable; a `let` array is immutable (see *Mutability and Copying*). Nested indexed assignment is supported.

```text
var arr = [1, 2, 3]
arr[0] = 99
print(arr[0]) // 99

var grid = [[1, 2], [3, 4]]
grid[0][1] = 99   // nested assignment
print(grid)       // [[1, 99], [3, 4]]
```

## 10. Dictionaries (Maps)
Created using braces `{}` with string keys. Accessed and mutated using square brackets `[]`. A `var` dictionary is mutable; a `let` dictionary is immutable (see *Mutability and Copying*).

Accessing a missing key returns `Null` instead of throwing an error. You can use the `??` operator to provide a fallback value.

```text
var user = {"name": "Jan", "age": 30}
print(user["name"]) // Jan

var role = user["role"] ?? "guest" // Returns "guest" because "role" is missing
print(role)

user["age"] = 31
print(user) // {"name": "Jan", "age": 31}
```

## 11. String Interpolation
Strings can contain expressions inside `{...}`. They will be evaluated and interpolated into the text.

```text
var name = "World"
var x = 5
print("Hello {name}! The result is {x + 5}") // Hello World! The result is 10
```

## 12. Error Handling (`execute` / `onError`)
Runtime errors (like division by zero, missing variables, or type mismatches) can be caught using the `execute` / `onError` expression. It attempts to run the first block. If an error occurs, it catches it, binds the error message to a variable, and runs the second block.

```text
var result = execute {
    10 / 0
} onError(err) {
    print("Caught an error: ", err)
    -1 // Return a fallback value
}
print(result) // -1
```

## 13. Built-in Functions and Extensions

### Global Functions
*   `print(...args)` - Prints arguments to the screen separated by spaces.
*   `input(prompt)` - Displays the prompt and waits for user input. Always returns a `String`.
*   `args()` - Returns an `Array` of `String` containing the command-line arguments passed to the script.
*   `exit(code)` - Terminates the program immediately with the given exit code (`Number`).
*   `shell(command)` - Executes a command in the system shell (`cmd` on Windows, `sh` on Unix) and returns the combined stdout/stderr output as a `String`.

### Extension Methods (Strings & Arrays)
Extension methods can be chained to values.

**String Methods:**
*   `.upper()` - Returns the string in uppercase.
*   `.lower()` - Returns the string in lowercase.
*   `.trim()` - Returns a new string with leading and trailing whitespace removed.
*   `.contains(substring)` - Returns `true` if the string contains the given substring.
*   `.replace(old, new)` - Returns a new string where all occurrences of `old` are replaced with `new`.
*   `.split(separator)` - Returns an `Array` of strings split by the separator.

**Array & String Shared Methods:**
*   `.length()` - Returns the length of a String (character count) or an Array (element count).
*   `.contains(element)` - Returns `true` if the Array/String contains the given element/substring.

**Array Methods:**
*   `.push(element)` - Adds an element to the end of the array (mutates the array in place). Only allowed on a mutable (`var`) array; calling it on an immutable (`let`) array is a runtime error. Extension methods also validate their argument count and report an error instead of failing when called with too few arguments.
*   `.join(separator)` - Joins all elements of the array into a single String, separated by the given separator.
*   `.map(fun)` - Returns a new array by applying the given function (lambda) to each element.
*   `.filter(fun)` - Returns a new array containing only elements for which the function returned `true`.

### Example Usage of Input and Extensions:
```text
var name = input("What is your name? ")
print("Hello, ", name, "!")

var age = input("Enter your age: ").asNumber()
print("Next year you will be ", age + 1, " years old.")

var nums = [1, 2, 3, 4, 5]
var evens = nums.filter(fun(x) { return x % 2 == 0 })
print("Even numbers: ", evens.join(", "))
```

## 14. File I/O (`use io`)
File operations are available by loading the `io` module. It provides a `file()` constructor which returns a `File` object (implemented as a Dictionary under the hood).

```text
use io

var f = file("output.txt")
if not f.exists() {
    f.write("New file\n")
}
f.append("Appending a new line\n")

var content = f.read()
print("File content:\n", content)
```
