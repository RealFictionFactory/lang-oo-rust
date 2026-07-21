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
    
    assert_eq!(tokens[3], Token::Number(5));
    assert_eq!(tokens[4], Token::NewLine);
}

// Test 3: Check if the parser correctly builds the AST for a variable declaration
#[test]
fn test_parser_var_decl() {
    let mut lex = Lexer::new("var y = 20");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    assert_eq!(ast.len(), 1);
}

// Test 4: Check if the interpreter does not throw errors on simple code
#[test]
fn test_interpreter_runs_without_error() {
    let code = "var a = 10\nvar b = 20\nprint(a + b)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_ok());
}

// Test 5: Check syntax errors (e.g., missing '=' and 'is' after variable name)
#[test]
fn test_parser_syntax_error() {
    let mut lex = Lexer::new("var x 10");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    
    assert!(result.is_err());
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
        Stmt::ExprStmt(Expr::If(_, if_body, else_body)) => {
            assert_eq!(if_body.len(), 1, "The 'if' block should have 1 statement");
            assert_eq!(else_body.len(), 1, "The 'else' block should have 1 statement");
        }
        _ => panic!("Expected a Stmt::ExprStmt(Expr::If) node, got something else!"),
    }
}

// Test 7: Check the correct execution of 'if/else' by the interpreter
#[test]
fn test_interpreter_if_else_execution() {
    let code = "var x = 10\nif x == 5 { var a = 1 } else { var b = 2 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
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
    let code = "var result = \"\"\nif 0 { result = \"fail\" } else { result = \"pass\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_ok(), "Interpreter should not error on numeric condition due to truthiness");
    assert_eq!(Environment::get(&env, "result").unwrap().value, crate::interpreter::Value::Str("pass".to_string()));
}

// Test 10: Check the execution of a 'for' loop using the iteration variable
#[test]
fn test_interpreter_for_loop_runs() {
    let code = "loop i from 1..3 { var x = i + 10 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
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
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_ok(), "Assignment to an existing variable should work");
    
    let var_info = Environment::get(&env, "x").expect("Variable x should exist");
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
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
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
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
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
    assert!(matches!(ast[0], Stmt::Let(..)), "Expected a Stmt::Let node");
}

// Test 15: Check operator precedence (multiplication before addition)
#[test]
fn test_interpreter_math_precedence() {
    let code = "var x = 2 + 3 * 4";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    let var_info = Environment::get(&env, "x").expect("Variable x should exist");
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
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_err(), "Division by zero should throw an error");
    assert_eq!(result.unwrap_err(), "Runtime error: Division by zero!");
}

// Test 17: Check comparison operators
#[test]
fn test_interpreter_comparison_operators() {
    let code = "var a = 5\nvar b = 10\nvar c = a < b\nvar d = a > b\nvar e = a != b";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "d").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(Environment::get(&env, "e").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 18: Check compound assignment operators (+=, -=)
#[test]
fn test_interpreter_compound_assignment() {
    let code = "var x = 5\nx += 10\nx -= 3";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    let var_info = Environment::get(&env, "x").expect("Variable x should exist");
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
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Bool(false));
}

// Test 20: Test array literals and indexing
#[test]
fn test_interpreter_arrays() {
    let code = "var arr = [10, 20, 30]\nvar x = arr[1]";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Number(20));
}

// Test 21: Test array mutation
#[test]
fn test_interpreter_array_mutation() {
    let code = "var arr = [1, 2, 3]\narr[1] = 99";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    let arr_val = &Environment::get(&env, "arr").unwrap().value;
    if let crate::interpreter::Value::Array(arr) = arr_val {
        assert_eq!(arr.borrow()[1], crate::interpreter::Value::Number(99));
    } else {
        panic!("Variable arr should be an array");
    }
}

// Test 22: Test user-defined functions
#[test]
fn test_interpreter_functions() {
    let code = "fun add(a, b) { return a + b }\nvar x = add(5, 10)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Number(15));
}

// Test 23: Test recursion (Fibonacci or Factorial)
#[test]
fn test_interpreter_recursion() {
    let code = "fun fib(n) { if n < 2 { return n } else { return fib(n - 1) + fib(n - 2) } }\nvar x = fib(10)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Number(55));
}

// Test 24: Test <= and >= operators
#[test]
fn test_comparison_leq_geq() {
    let code = "var a = 5\nvar b = 5\nvar c = 10\nvar eq = a <= b\nvar gt = c >= b";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "eq").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "gt").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 25: Test break statement
#[test]
fn test_loop_break() {
    let code = "var sum = 0\nloop i from 1..5 {\nif i == 3 {\nbreak\n}\nsum += i\n}";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "sum").unwrap().value, crate::interpreter::Value::Number(3));
}

// Test 26: Test continue statement
#[test]
fn test_loop_continue() {
    let code = "var sum = 0\nloop i from 1..5 {\nif i == 3 {\ncontinue\n}\nsum += i\n}";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "sum").unwrap().value, crate::interpreter::Value::Number(7));
}

// Test 27: Test modulo operator
#[test]
fn test_modulo_operator() {
    let code = "var a = 10 % 3\nvar b = 10 % 2";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Number(1));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Number(0));
}

// Test 28: Test string interpolation
#[test]
fn test_string_interpolation() {
    let code = "var name = \"Ó\"\nvar x = 5\nvar msg = \"Jezyk {name} ma wartosc {x + 10}\"";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "msg").unwrap().value, crate::interpreter::Value::Str("Jezyk Ó ma wartosc 15".to_string()));
}

// Test 29: Test array and string methods (push, length)
#[test]
fn test_builtin_methods() {
    let code = "var arr = [1, 2, 3]\narr.push(4)\nvar l1 = arr.length()\nvar s = \"hello\"\nvar l2 = s.length()";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "l1").unwrap().value, crate::interpreter::Value::Number(4));
    assert_eq!(Environment::get(&env, "l2").unwrap().value, crate::interpreter::Value::Number(5));
}

// Test 30: Test type promotion (Numeric + Decimal = Decimal)
#[test]
fn test_type_promotion() {
    let code = "var a = 5\nvar b = 2.5\nvar c = a + b\nvar d = 10 / 3";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Decimal(7.5));
    assert_eq!(Environment::get(&env, "d").unwrap().value, crate::interpreter::Value::Number(3));
}

// Test 31: Test type annotations and default values
#[test]
fn test_type_annotations() {
    let code = "var a is Number\nvar b is String\nvar c is Bool\nlet d is Decimal = 3.14";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Number(0));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Str("".to_string()));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(Environment::get(&env, "d").unwrap().value, crate::interpreter::Value::Decimal(3.14));
}

// Test 32: Test truthiness (0 and empty string are false)
#[test]
fn test_truthiness() {
    let code = "var zero = 0\nvar empty = \"\"\nvar result = \"\"\nif zero { result = \"fail\" } else { result = \"pass\" }\nif empty { result = \"fail\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "result").unwrap().value, crate::interpreter::Value::Str("pass".to_string()));
}

// Test 33: Test method calls (arr.push)
#[test]
fn test_method_calls() {
    let code = "var arr = [1, 2]\narr.push(3)\nvar l = arr.length()";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    let arr_val = &Environment::get(&env, "arr").unwrap().value;
    if let crate::interpreter::Value::Array(arr) = arr_val {
        assert_eq!(arr.borrow().len(), 3);
    } else {
        panic!("Variable arr should be an array");
    }
    
    assert_eq!(Environment::get(&env, "l").unwrap().value, crate::interpreter::Value::Number(3));
}

// Test 34: Test execute/onError error handling
#[test]
fn test_execute_on_error() {
    let code = "var x = execute { 10 / 0 } onError(err) { print(\"Caught: \", err) 99 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Number(99));
}

// Test 35: Test if as an expression
#[test]
fn test_if_as_expression() {
    let code = "var x = 5\nvar y = if x > 3 { \"big\" } else { \"small\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "y").unwrap().value, crate::interpreter::Value::Str("big".to_string()));
}

// Test 36: Test 'return if {} else {}' inside a function
#[test]
fn test_return_if_expression() {
    let code = "fun check(n) { return if n > 0 { \"positive\" } else { \"zero or negative\" } }\nvar x = check(5)\nvar y = check(-2)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Str("positive".to_string()));
    assert_eq!(Environment::get(&env, "y").unwrap().value, crate::interpreter::Value::Str("zero or negative".to_string()));
}

// Test 37: Test logical operators (and, or)
#[test]
fn test_logical_operators() {
    let code = "var a = true and false\nvar b = true or false\nvar c = (5 > 3) and (10 > 5)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 38: Test short-circuit evaluation for 'and'
#[test]
fn test_short_circuit_and() {
    let code = "var x = false and (10 / 0 == 1)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_ok(), "Short-circuit 'and' should not evaluate the right side");
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Bool(false));
}

// Test 39: Test short-circuit evaluation for 'or'
#[test]
fn test_short_circuit_or() {
    let code = "var x = true or (10 / 0 == 1)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_ok(), "Short-circuit 'or' should not evaluate the right side");
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 40: Test unary 'not' operator with truthiness
#[test]
fn test_unary_not() {
    let code = "var a = not true\nvar b = not false\nvar c = not 0\nvar d = not \"\"";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "d").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 41: Test unary minus operator
#[test]
fn test_unary_minus() {
    let code = "var a = -5\nvar b = 10 + -3\nvar c = -(-5)\nvar d = -3.14";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Number(-5));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Number(7));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Number(5));
    assert_eq!(Environment::get(&env, "d").unwrap().value, crate::interpreter::Value::Decimal(-3.14));
}

// Test 42: Test 'loop in' for array iteration
#[test]
fn test_loop_in_array() {
    let code = "var arr = [10, 20, 30]\nvar sum = 0\nloop x in arr { sum += x }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "sum").unwrap().value, crate::interpreter::Value::Number(60));
}

// Test 43: Test infinite 'loop {}' with a 'break'
#[test]
fn test_loop_block_break() {
    let code = "var i = 0\nvar count = 0\nloop {\nif i >= 5 { break }\ncount += 1\ni += 1\n}";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "count").unwrap().value, crate::interpreter::Value::Number(5));
}

// Test 44: Test 'until' at the top (acts like 'while not')
#[test]
fn test_until_at_top() {
    let code = "var i = 10\nvar count = 0\nloop {\nuntil (i > 5)\ncount += 1\n}";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "count").unwrap().value, crate::interpreter::Value::Number(0));
}

// Test 45: Test 'until' at the bottom (acts like 'do-while not')
#[test]
fn test_until_at_bottom() {
    let code = "var i = 10\nvar count = 0\nloop {\ncount += 1\ni += 1\nuntil (i > 5)\n}";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "count").unwrap().value, crate::interpreter::Value::Number(1));
}

// Test 46: Test dictionary creation and access
#[test]
fn test_dictionary_access() {
    let code = "var user = {\"name\": \"Jan\", \"age\": 30}\nvar n = user[\"name\"]\nvar a = user[\"age\"]";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "n").unwrap().value, crate::interpreter::Value::Str("Jan".to_string()));
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Number(30));
}

// Test 47: Test dictionary mutation
#[test]
fn test_dictionary_mutation() {
    let code = "var config = {\"debug\": false}\nconfig[\"debug\"] = true";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    let config_val = &Environment::get(&env, "config").unwrap().value;
    if let crate::interpreter::Value::Dict(map) = config_val {
        assert_eq!(map.borrow().get("debug").unwrap(), &crate::interpreter::Value::Bool(true));
    } else {
        panic!("Variable config should be a dictionary");
    }
}

// Test 48: Test missing key returns Null (for ?? operator)
#[test]
fn test_dictionary_missing_key_returns_null() {
    let code = "var user = {\"name\": \"Jan\"}\nvar x = user[\"age\"]";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Null);
}

// Test 49: Test dictionary string representation
#[test]
fn test_dictionary_to_string() {
    let code = "var user = {\"name\": \"Jan\", \"age\": 30}\nvar s = \"\"\nloop key in [\"name\", \"age\"] {\n s += \"{key}: {user[key]} \" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "s").unwrap().value, crate::interpreter::Value::Str("name: Jan age: 30 ".to_string()));
}

// Test 50: Test correct type annotation on declaration
#[test]
fn test_type_check_correct_decl() {
    let code = "var x is Number = 10\nvar y is String = \"hello\"\nvar z is Bool = true";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_ok(), "Correct type annotations should not fail");
    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Number(10));
}

// Test 51: Test type mismatch on declaration
#[test]
fn test_type_check_mismatch_decl() {
    let code = "var x is Number = \"string\"";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_err(), "Should fail due to type mismatch");
    assert_eq!(result.unwrap_err(), "Type mismatch: expected 'Number', got String");
}

// Test 52: Test type mismatch on assignment
#[test]
fn test_type_check_mismatch_assign() {
    let code = "var x is Number = 10\nx = true";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_err(), "Should fail when assigning wrong type to typed variable");
    assert_eq!(result.unwrap_err(), "Type mismatch: cannot assign Bool to variable of type Number");
}

// Test 53: Test default value for Array type
#[test]
fn test_type_check_default_array() {
    let code = "var arr is Array\narr.push(99)\nvar l = arr.length()";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "l").unwrap().value, crate::interpreter::Value::Number(1));
}

// Test 54: Test default value for Dict type
#[test]
fn test_type_check_default_dict() {
    let code = "var d is Dict\nd[\"key\"] = \"value\"";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    let d_val = &Environment::get(&env, "d").unwrap().value;
    if let crate::interpreter::Value::Dict(map) = d_val {
        assert_eq!(map.borrow().get("key").unwrap(), &crate::interpreter::Value::Str("value".to_string()));
    } else {
        panic!("Variable d should be a Dict");
    }
}

// Test 55: Test match expression with literals
#[test]
fn test_match_literals() {
    let code = "var x = 2\nvar name = match x { 0 -> \"zero\" 1 -> \"one\" _ -> \"many\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "name").unwrap().value, crate::interpreter::Value::Str("many".to_string()));
}

// Test 56: Test match expression with block bodies
#[test]
fn test_match_block_bodies() {
    let code = "var x = 1\nvar result = match x { 0 -> { var a = 10 a + 5 } 1 -> { var b = 20 b * 2 } _ -> 0 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "result").unwrap().value, crate::interpreter::Value::Number(40));
}

// Test 57: Test match with string literals
#[test]
fn test_match_strings() {
    let code = "var lang = \"Ó\"\nvar is_cool = match lang { \"Ó\" -> true \"Java\" -> false _ -> false }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "is_cool").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 58: Test match exhaustion (no matching arm)
#[test]
fn test_match_exhaustion_fails() {
    let code = "var x = 99\nvar y = match x { 0 -> \"zero\" 1 -> \"one\" }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    
    assert!(result.is_err(), "Match should fail if no arm matches and there is no wildcard");
    assert_eq!(result.unwrap_err(), "Match expression exhausted with no matching arm");
}

// Test 59: Test nullish coalescing (??) operator
#[test]
fn test_nullish_coalescing() {
    let code = "var user = {\"name\": \"Jan\"}\nvar name = user[\"name\"] ?? \"Anonymous\"\nvar age = user[\"age\"] ?? 18";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "name").unwrap().value, crate::interpreter::Value::Str("Jan".to_string()));
    assert_eq!(Environment::get(&env, "age").unwrap().value, crate::interpreter::Value::Number(18));
}

// Test 60: Test lambda assignment and call
#[test]
fn test_lambda_assignment() {
    let code = "var double = fun(x) { return x * 2 }\nvar res = double(5)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "res").unwrap().value, crate::interpreter::Value::Number(10));
}

// Test 61: Test closures (function capturing state)
#[test]
fn test_closures() {
    let code = "
        fun make_counter() {
            var count = 0
            return fun() {
                count = count + 1
                return count
            }
        }
        var c = make_counter()
        var r1 = c()
        var r2 = c()
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "r1").unwrap().value, crate::interpreter::Value::Number(1));
    assert_eq!(Environment::get(&env, "r2").unwrap().value, crate::interpreter::Value::Number(2));
}

// Test 62: Test passing functions as arguments (Higher-order functions)
#[test]
fn test_higher_order_functions() {
    let code = "
        fun apply(func, val) {
            return func(val)
        }
        var result = apply(fun(x) { return x + 10 }, 5)
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "result").unwrap().value, crate::interpreter::Value::Number(15));
}

// Test 63: Test string methods (trim, contains, replace, split)
#[test]
fn test_string_methods() {
    let code = "
        var s = \"  hello world  \"
        var t = s.trim()
        var c1 = s.contains(\"world\")
        var c2 = s.contains(\"Ó\")
        var r = s.replace(\"world\", \"Ó\")
        var p = \"a,b,c\".split(\",\")
        var p0 = p[0]
        var p1 = p[1]
        var p2 = p[2]
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "t").unwrap().value, crate::interpreter::Value::Str("hello world".to_string()));
    assert_eq!(Environment::get(&env, "c1").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "c2").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(Environment::get(&env, "r").unwrap().value, crate::interpreter::Value::Str("  hello Ó  ".to_string()));
    assert_eq!(Environment::get(&env, "p0").unwrap().value, crate::interpreter::Value::Str("a".to_string()));
    assert_eq!(Environment::get(&env, "p1").unwrap().value, crate::interpreter::Value::Str("b".to_string()));
    assert_eq!(Environment::get(&env, "p2").unwrap().value, crate::interpreter::Value::Str("c".to_string()));
}

// Test 64: Test array methods (contains, join)
#[test]
fn test_array_methods() {
    let code = "
        var arr = [\"apple\", \"banana\", \"cherry\"]
        var c1 = arr.contains(\"banana\")
        var c2 = arr.contains(\"orange\")
        var j = arr.join(\", \")
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "c1").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "c2").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(Environment::get(&env, "j").unwrap().value, crate::interpreter::Value::Str("apple, banana, cherry".to_string()));
}

// Test 65: Test array.map(fun)
#[test]
fn test_array_map() {
    let code = "
        var nums = [1, 2, 3, 4, 5]
        var doubled = nums.map(fun(x) { return x * 2 })
        var d0 = doubled[0]
        var d4 = doubled[4]
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "d0").unwrap().value, crate::interpreter::Value::Number(2));
    assert_eq!(Environment::get(&env, "d4").unwrap().value, crate::interpreter::Value::Number(10));
}

// Test 66: Test array.filter(fun)
#[test]
fn test_array_filter() {
    let code = "
        var nums = [1, 2, 3, 4, 5, 6]
        var evens = nums.filter(fun(x) { return x % 2 == 0 })
        var e0 = evens[0]
        var e1 = evens[1]
        var e2 = evens[2]
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "e0").unwrap().value, crate::interpreter::Value::Number(2));
    assert_eq!(Environment::get(&env, "e1").unwrap().value, crate::interpreter::Value::Number(4));
    assert_eq!(Environment::get(&env, "e2").unwrap().value, crate::interpreter::Value::Number(6));
}

// Test 67: Test array.push after Rc<RefCell> refactor
#[test]
fn test_array_push_rc_refcell() {
    let code = "
        var arr = [1, 2, 3]
        arr.push(4)
        var l = arr.length()
        var v3 = arr[3]
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "l").unwrap().value, crate::interpreter::Value::Number(4));
    assert_eq!(Environment::get(&env, "v3").unwrap().value, crate::interpreter::Value::Number(4));
}

// Test 68: Test File I/O operations (requires 'use io')
#[test]
fn test_file_operations() {
    let filename = "oo_test_file_tmp.txt";
    let _ = std::fs::remove_file(filename); // Cleanup before test

    let code = format!("
        use io
        
        var f = file(\"{}\")
        var exists_before = f.exists()
        f.write(\"Line 1\\n\")
        f.append(\"Line 2\\n\")
        var content = f.read()
        var exists_after = f.exists()
    ", filename);
    
    let mut lex = Lexer::new(&code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    
    assert_eq!(Environment::get(&env, "exists_before").unwrap().value, crate::interpreter::Value::Bool(false));
    assert_eq!(Environment::get(&env, "exists_after").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "content").unwrap().value, crate::interpreter::Value::Str("Line 1\nLine 2\n".to_string()));
    
    let _ = std::fs::remove_file(filename); // Cleanup after test
}
