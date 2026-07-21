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
*   `Array` - Array of elements. Passed by reference (mutable inside functions).
*   `Dict` - A collection of key-value pairs where keys are Strings. Created using braces `{"key": value}`. Passed by reference.
*   `Null` - Absence of a value (returned e.g., by functions without a `return` statement or missing dictionary keys).

## 3. Variables and Constants
Declarations use the `var` (mutable) and `let` (immutable) keywords. You can specify the type using `is Type`, which assigns a default value (`0` for numbers, `false` for Bool, `""` for String, `[]` for Array, `{}` for Dict).

```text
var x = 10
let pi = 3.14
var name is String  // defaults to ""
var arr is Array // defaults to []
```

### Runtime Type Checking
If a type is explicitly specified, the language enforces it at runtime. Attempting to assign a value of the wrong type to a typed variable will result in a runtime error.

```text
var age is Number = 20
age = "twenty" // Runtime error: Type mismatch: cannot assign String to variable of type Number
```

## 4. Operators
*   **Arithmetic:** `+`, `-`, `*`, `/`, `%` (modulo).
*   **Unary:** `-` (negation, e.g., `-5`), `not` (logical negation, e.g., `not true`).
*   **Logical:** `and`, `or` (support short-circuit evaluation).
*   **Comparison:** `==`, `!=`, `>`, `<`, `>=`, `<=`.
*   **Assignment:** `=`, `+=`, `-=`.
*   **Nullish Coalescing:** `??` (returns the left value if it is not `Null`, otherwise evaluates and returns the right value).

*Concatenation:* The `+` operator concatenates strings. If you concatenate a String with a Number/Decimal, the number is automatically converted to text.

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
Defined using the `fun` keyword. They can return a value using `return`. They support recursion and have their own local scope. Functions are First-Class Citizens, meaning they can be assigned to variables (lambdas), passed as arguments, and returned from other functions (closures).

```text
fun add(a, b) {
  return a + b
}

// Lambda assigned to a variable
var double = fun(x) { return x * 2 }

// Closure capturing state
fun make_counter() {
    var count = 0
    return fun() {
        count = count + 1
        return count
    }
}
var c = make_counter()
print(c()) // 1
print(c()) // 2
```

## 9. Arrays
Created using square brackets `[]`. Indexed from `0`. Passed by reference.

```text
var arr = [1, 2, 3]
arr[0] = 99
print(arr[0]) // 99
```

## 10. Dictionaries (Maps)
Created using braces `{}` with string keys. Accessed and mutated using square brackets `[]`. Passed by reference.

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
*   `.push(element)` - Adds an element to the end of the array (mutates the array in place).
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
