// src/tests.rs

/// Unit tests for the "Ó" programming language.
/// Covers lexer, parser, and interpreter functionality.
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;
use crate::interpreter::Environment;
use crate::ast::{Expr, Stmt};

// Test 1: Check if the lexer correctly splits the code into tokens
#[test]
fn test_lexer_basic() {
    let mut lex = Lexer::new("var x = 10");
    let tokens = lex.tokenize();

    assert_eq!(tokens[0], Token::Var);
    assert_eq!(tokens[1], Token::Ident("x".to_string()));
    assert_eq!(tokens[2], Token::Assign);
    assert_eq!(tokens[3], Token::Number(10));
}

// Test 2: Check if comments are correctly ignored by the lexer
#[test]
fn test_lexer_comments() {
    let mut lex = Lexer::new("var x = 5 // this is a comment\n");
    let tokens = lex.tokenize();
    
    // We expect that after the number 5, a newline appears immediately, and the comment is gone
    assert_eq!(tokens[3], Token::Number(5));
    assert_eq!(tokens[4], Token::NewLine);
}

// Test 3: Check if the parser correctly builds the AST for a variable declaration
#[test]
fn test_parser_var_decl() {
    let mut lex = Lexer::new("var y = 20");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap(); // unwrap() is ok in tests; if it throws an error, the test fails

    assert_eq!(ast.len(), 1); // There should be one statement
    // We can also check if it's the correct node (requires importing Stmt from ast)
    // assert_eq!(ast[0], crate::ast::Stmt::VarDecl("y".to_string(), crate::ast::Expr::Number(20)));
}

// Test 4: Check if the interpreter does not throw errors on simple code
#[test]
fn test_interpreter_runs_without_error() {
    let code = "var a = 10\nvar b = 20\nprint(a + b)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok()); // Check if execution ended successfully
}

// Test 5: Check syntax errors (e.g., missing '=' and 'is' after variable name)
#[test]
fn test_parser_syntax_error() {
    let mut lex = Lexer::new("var x 10");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    
    assert!(result.is_err()); // Expect the parser to return an error (Err)
    assert_eq!(result.unwrap_err(), "Variable declaration must have a type 'is Type' or an initial value '='");
}

// Test 6: Check if the parser correctly builds the AST for 'if' and 'else'
#[test]
fn test_parser_if_else_ast() {
    let code = "if x == 5 { print(1) } else { print(2) }";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        // CHANGE: Now 'if' is wrapped in ExprStmt
        Stmt::ExprStmt(Expr::If(_, if_body, else_body)) => {
            assert_eq!(if_body.len(), 1, "The 'if' block should have 1 statement");
            assert_eq!(else_body.len(), 1, "The 'else' block should have 1 statement");
        }
        _ => panic!("Expected a Stmt::ExprStmt(Expr::If) node, got something else!"),
    }
}

// Test 7: Check the correct execution of 'if/else' by the interpreter
// Note that the code in the test does not have 'print', we just check if 
// executing code that enters the 'else' block does not cause errors.
#[test]
fn test_interpreter_if_else_execution() {
    // x is 10, so the condition x == 5 is false. It should enter 'else'.
    let code = "var x = 10\nif x == 5 { var a = 1 } else { var b = 2 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok(), "Interpreter should execute the code without errors");
}

// Test 8: Check if the 'loop' is built correctly
#[test]
fn test_parser_for_loop_ast() {
    let code = "loop i from 1..5 { print(i) }";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        // Pattern: Loop(variable_name, start, end, code_block)
        Stmt::Loop(var_name, _, _, body) => {
            assert_eq!(var_name, "i", "The iteration variable should be named 'i'");
            assert_eq!(body.len(), 1, "The loop block should have 1 statement");
        }
        _ => panic!("Expected a Stmt::Loop node, got something else!"),
    }
}

// Test 9: Check if the interpreter uses truthiness (0 is false, 5 is true)
#[test]
fn test_interpreter_if_truthiness() {
    // 0 is false, so it should enter the else block
    let code = "var result = \"\"\nif 0 { result = \"fail\" } else { result = \"pass\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok(), "Interpreter should not error on numeric condition due to truthiness");
    assert_eq!(env.get("result").unwrap().value, crate::interpreter::Value::Str("pass".to_string()));
}

// Test 10: Check the execution of a 'for' loop using the iteration variable
#[test]
fn test_interpreter_for_loop_runs() {
    // A loop that simply performs a mathematical operation. 
    // We don't print to the screen to avoid cluttering the console during tests.
    let code = "loop i from 1..3 { var x = i + 10 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok(), "The loop should execute without errors");
}

// Test 11: Check if a variable can be overwritten (Assign)
#[test]
fn test_interpreter_variable_assignment() {
    let code = "var x = 5\nx = 20";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok(), "Assignment to an existing variable should work");
    
    // We go into VarInfo and check the .value field
    let var_info = env.get("x").expect("Variable x should exist");
    assert_eq!(var_info.value, crate::interpreter::Value::Number(20));
    assert!(!var_info.is_const, "x should not be a constant");
}

// Test 12: Check if assigning to an undeclared variable throws an error
#[test]
fn test_interpreter_assign_undeclared_fails() {
    let code = "y = 10";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_err(), "An error should occur when assigning to an undeclared variable");
    assert_eq!(
        result.unwrap_err(), 
        "Variable 'y' is not declared. Use 'var' or 'let'."
    );
}

// Test 13: Check if attempting to change a constant (let) results in an error
#[test]
fn test_interpreter_const_reassignment_fails() {
    let code = "let y = 10\ny = 20";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_err(), "An error should occur when changing a constant");
    assert_eq!(
        result.unwrap_err(), 
        "Cannot change value of constant 'y'"
    );
}

// Test 14: Check if the parser correctly builds the AST for a 'let' constant
#[test]
fn test_parser_let_decl() {
    let mut lex = Lexer::new("let y = 20");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    assert_eq!(ast.len(), 1);
    // Check if it's a Let node
    assert!(matches!(ast[0], Stmt::Let(..)), "Expected a Stmt::Let node");
}

// Test 15: Check operator precedence (multiplication before addition)
#[test]
fn test_interpreter_math_precedence() {
    // 2 + 3 * 4 should equal 14, not 20
    let code = "var x = 2 + 3 * 4";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    let var_info = env.get("x").expect("Variable x should exist");
    assert_eq!(var_info.value, crate::interpreter::Value::Number(14));
}

// Test 16: Check if division by zero throws an error
#[test]
fn test_interpreter_division_by_zero() {
    let code = "var y = 10 / 0";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_err(), "Division by zero should throw an error");
    assert_eq!(result.unwrap_err(), "Runtime error: Division by zero!");
}

// Test 17: Check comparison operators
#[test]
fn test_interpreter_comparison_operators() {
    // 5 < 10 -> true
    // 5 > 10 -> false
    // 5 != 10 -> true
    let code = "var a = 5\nvar b = 10\nvar c = a < b\nvar d = a > b\nvar e = a != b";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("c").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(env.get("d").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(env.get("e").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 18: Check compound assignment operators (+=, -=)
#[test]
fn test_interpreter_compound_assignment() {
    // var x = 5
    // x += 10  (x should be 15)
    // x -= 3   (x should be 12)
    let code = "var x = 5\nx += 10\nx -= 3";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    let var_info = env.get("x").expect("Variable x should exist");
    assert_eq!(var_info.value, crate::interpreter::Value::Number(12));
}

// Test 19: Check boolean literals (true/false)
#[test]
fn test_interpreter_boolean_literals() {
    let code = "var a = true\nvar b = false";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("a").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(env.get("b").unwrap().value, crate::interpreter::Value::Bool(false));
}

// Test 20: Test array literals and indexing
#[test]
fn test_interpreter_arrays() {
    let code = "var arr = [10, 20, 30]\nvar x = arr[1]";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("x").unwrap().value, crate::interpreter::Value::Number(20));
}

// Test 21: Test array mutation
#[test]
fn test_interpreter_array_mutation() {
    let code = "var arr = [1, 2, 3]\narr[1] = 99";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    let arr_val = &env.get("arr").unwrap().value;
    if let crate::interpreter::Value::Array(arr) = arr_val {
        assert_eq!(arr[1], crate::interpreter::Value::Number(99));
    } else {
        panic!("Variable arr should be an array");
    }
}

// Test 22: Test user-defined functions
#[test]
fn test_interpreter_functions() {
    let code = "func add(a, b) { return a + b }\nvar x = add(5, 10)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("x").unwrap().value, crate::interpreter::Value::Number(15));
}

// Test 23: Test recursion (Fibonacci or Factorial)
#[test]
fn test_interpreter_recursion() {
    let code = "func fib(n) { if n < 2 { return n } else { return fib(n - 1) + fib(n - 2) } }\nvar x = fib(10)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    // fib(10) = 55
    assert_eq!(env.get("x").unwrap().value, crate::interpreter::Value::Number(55));
}

// Test 24: Test <= and >= operators
#[test]
fn test_comparison_leq_geq() {
    let code = "var a = 5\nvar b = 5\nvar c = 10\nvar eq = a <= b\nvar gt = c >= b";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("eq").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(env.get("gt").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 25: Test break statement
#[test]
fn test_loop_break() {
    // Sum numbers 1..5, but break when i == 3
    // So sum should be 1 + 2 = 3
    let code = "var sum = 0\nloop i from 1..5 {\nif i == 3 {\nbreak\n}\nsum += i\n}";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("sum").unwrap().value, crate::interpreter::Value::Number(3));
}

// Test 26: Test continue statement
#[test]
fn test_loop_continue() {
    // Sum numbers 1..5, but skip when i == 3
    // So sum should be 1 + 2 + 4 = 7
    let code = "var sum = 0\nloop i from 1..5 {\nif i == 3 {\ncontinue\n}\nsum += i\n}";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("sum").unwrap().value, crate::interpreter::Value::Number(7));
}

// Test 27: Test modulo operator
#[test]
fn test_modulo_operator() {
    // 10 % 3 = 1
    // 10 % 2 = 0 (even)
    let code = "var a = 10 % 3\nvar b = 10 % 2";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("a").unwrap().value, crate::interpreter::Value::Number(1));
    assert_eq!(env.get("b").unwrap().value, crate::interpreter::Value::Number(0));
}

// Test 28: Test string interpolation
#[test]
fn test_string_interpolation() {
    let code = "var name = \"Ó\"\nvar x = 5\nvar msg = \"Jezyk {name} ma wartosc {x + 10}\"";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("msg").unwrap().value, crate::interpreter::Value::Str("Jezyk Ó ma wartosc 15".to_string()));
}

// Test 29: Test array and string methods (push, length)
#[test]
fn test_builtin_methods() {
    let code = "var arr = [1, 2, 3]\narr.push(4)\nvar l1 = arr.length()\nvar s = \"hello\"\nvar l2 = s.length()";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("l1").unwrap().value, crate::interpreter::Value::Number(4));
    assert_eq!(env.get("l2").unwrap().value, crate::interpreter::Value::Number(5));
}

// Test 30: Test type promotion (Numeric + Decimal = Decimal)
#[test]
fn test_type_promotion() {
    let code = "var a = 5\nvar b = 2.5\nvar c = a + b\nvar d = 10 / 3";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    // 5 + 2.5 = 7.5 (Decimal)
    assert_eq!(env.get("c").unwrap().value, crate::interpreter::Value::Decimal(7.5));
    // 10 / 3 = 3 (Numeric, because both are integers!)
    assert_eq!(env.get("d").unwrap().value, crate::interpreter::Value::Number(3));
}

// Test 31: Test type annotations and default values
#[test]
fn test_type_annotations() {
    let code = "var a is Number\nvar b is String\nvar c is Bool\nlet d is Decimal = 3.14";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("a").unwrap().value, crate::interpreter::Value::Number(0));
    assert_eq!(env.get("b").unwrap().value, crate::interpreter::Value::Str("".to_string()));
    assert_eq!(env.get("c").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(env.get("d").unwrap().value, crate::interpreter::Value::Decimal(3.14));
}

// Test 32: Test truthiness (0 and empty string are false)
#[test]
fn test_truthiness() {
    let code = "var zero = 0\nvar empty = \"\"\nvar result = \"\"\nif zero { result = \"fail\" } else { result = \"pass\" }\nif empty { result = \"fail\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("result").unwrap().value, crate::interpreter::Value::Str("pass".to_string()));
}

// Test 33: Test method calls (arr.push)
#[test]
fn test_method_calls() {
    let code = "var arr = [1, 2]\narr.push(3)\nvar l = arr.length()";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    let arr_val = &env.get("arr").unwrap().value;
    if let crate::interpreter::Value::Array(arr) = arr_val {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Variable arr should be an array");
    }
    
    assert_eq!(env.get("l").unwrap().value, crate::interpreter::Value::Number(3));
}

// Test 34: Test execute/onError error handling
#[test]
fn test_execute_on_error() {
    let code = "var x = execute { 10 / 0 } onError(err) { print(\"Caught: \", err) 99 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    // x should be 99, because the error was caught and 99 was the last expression in onError
    assert_eq!(env.get("x").unwrap().value, crate::interpreter::Value::Number(99));
}

// Test 35: Test if as an expression
#[test]
fn test_if_as_expression() {
    let code = "var x = 5\nvar y = if x > 3 { \"big\" } else { \"small\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("y").unwrap().value, crate::interpreter::Value::Str("big".to_string()));
}

// Test 36: Test 'return if {} else {}' inside a function
#[test]
fn test_return_if_expression() {
    let code = "func check(n) { return if n > 0 { \"positive\" } else { \"zero or negative\" } }\nvar x = check(5)\nvar y = check(-2)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    assert_eq!(env.get("x").unwrap().value, crate::interpreter::Value::Str("positive".to_string()));
    assert_eq!(env.get("y").unwrap().value, crate::interpreter::Value::Str("zero or negative".to_string()));
}
