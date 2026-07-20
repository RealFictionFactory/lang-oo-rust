# Ó Language - Syntax Documentation

"Ó" is a simple, dynamically typed programming language with a scripting nature. It was designed with readability in mind, avoiding boilerplate (like semicolons) and sounding natural.

## 1. Basics
*   **No Semicolons:** End of line means end of statement. Blank lines are ignored.
*   **Comments:** Start with `//` and continue to the end of the line.
*   **Code Blocks:** Enclosed in braces `{ ... }`. The opening brace can be on a new line.

## 2. Data Types
The language has built-in types that can be optionally specified during declaration:
*   `Number` - 64-bit integer.
*   `Decimal` - 64-bit floating-point number.
*   `String` - Text enclosed in double quotes.
*   `Bool` - Logical value `true` or `false`.
*   `Array` - Array of elements.
*   `Dict` - A collection of key-value pairs where keys are Strings. Created using braces `{"key": value}`.
*   `Null` - Absence of a value (returned e.g., by functions without a `return` statement).

## 3. Variables and Constants
Declarations use the `var` (mutable) and `let` (immutable) keywords. You can specify the type using `is Type`, which assigns a default value (`0` for numbers, `false` for Bool, `""` for String).

```text
var x = 10
let pi = 3.14
var name is String  // defaults to ""
var counter is Number // defaults to 0
```

## 4. Operators
*   **Arithmetic:** `+`, `-`, `*`, `/`, `%` (modulo).
*   **Unary:** `-` (negation, e.g., `-5`), `not` (logical negation, e.g., `not true`).
*   **Logical:** `and`, `or` (support short-circuit evaluation).
*   **Comparison:** `==`, `!=`, `>`, `<`, `>=`, `<=`.
*   **Assignment:** `=`, `+=`, `-=`.

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

## 6. Loops (`loop`)
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

## 7. Functions
Defined using the `fun` keyword. They can return a value using `return`. They support recursion and have their own local scope (variables inside a function do not overwrite global variables).

```text
fun add(a, b) {
  return a + b
}

fun factorial(n) {
  if n <= 1 {
    return 1
  }
  return n * factorial(n - 1)
}

print(factorial(5)) // 120
```

## 8. Arrays
Created using square brackets `[]`. Indexed from `0`.

```text
var arr = [1, 2, 3]
arr[0] = 99
print(arr[0]) // 99
```

## 9. Dictionaries (Maps)
Created using braces `{}` with string keys. Accessed and mutated using square brackets `[]`.

```text
var user = {"name": "Jan", "age": 30}
print(user["name"]) // Jan
user["age"] = 31
print(user) // {"name": "Jan", "age": 31}
```

## 10. String Interpolation
Strings can contain expressions inside `{...}`. They will be evaluated and interpolated into the text.

```text
var name = "World"
var x = 5
print("Hello {name}! The result is {x + 5}") // Hello World! The result is 10
```

## 11. Error Handling (`execute` / `onError`)
Runtime errors (like division by zero or missing variables) can be caught using the `execute` / `onError` expression. It attempts to run the first block. If an error occurs, it catches it, binds the error message to a variable, and runs the second block.

```text
var result = execute {
    10 / 0
} onError(err) {
    print("Caught an error: ", err)
    -1 // Return a fallback value
}
print(result) // -1
```

## 12. Built-in Functions and Extensions

### Global Functions
*   `print(...args)` - Prints arguments to the screen separated by spaces.
*   `input(prompt)` - Displays the prompt and waits for user input. Always returns a `String`.

### Extension Methods
Extension methods can be chained to values.

**String Conversions:**
*   `.asNumber()` - Converts a String to a `Number` (integer).
*   `.asDecimal()` - Converts a String to a `Decimal` (float).
*   `.asBoolean()` - Converts a String to a `Bool` (recognizes "true"/"1" as true, "false"/"0" as false).

**String Methods:**
*   `.upper()` - Returns the string in uppercase.
*   `.lower()` - Returns the string in lowercase.

**Array & String Shared Methods:**
*   `.length()` - Returns the length of a String (character count) or an Array (element count).

**Array Methods:**
*   `.push(element)` - Adds an element to the end of the array (mutates the array in place).

### Example Usage of Input and Extensions:
```text
var name = input("What is your name? ")
print("Hello, ", name, "!")

var age = input("Enter your age: ").asNumber()
print("Next year you will be ", age + 1, " years old.")

let shout = input("Say something quietly: ").upper()
print("SHOUTING: ", shout)
```
