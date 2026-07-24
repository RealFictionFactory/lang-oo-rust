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
    if let crate::interpreter::Value::Array(arr, _) = arr_val {
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
    if let crate::interpreter::Value::Array(arr, _) = arr_val {
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
    if let crate::interpreter::Value::Dict(map, _) = config_val {
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
    if let crate::interpreter::Value::Dict(map, _) = d_val {
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

// Test 69: A map() callback may assign to a variable in the enclosing scope.
// Regression: the extension lookup used to hold a borrow on the environment while
// running the callback, so any assignment panicked with "RefCell already borrowed".
#[test]
fn test_map_callback_assigns_to_outer_variable() {
    let code = "
        var total = 0
        var nums = [1, 2, 3]
        var same = nums.map(fun(x) { total = total + x
            return x })
        var s0 = same[0]
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "total").unwrap().value, crate::interpreter::Value::Number(6));
    assert_eq!(Environment::get(&env, "s0").unwrap().value, crate::interpreter::Value::Number(1));
}

// Test 70: The same applies to filter() callbacks.
#[test]
fn test_filter_callback_assigns_to_outer_variable() {
    let code = "
        var seen = 0
        var nums = [1, 2, 3, 4]
        var evens = nums.filter(fun(x) { seen += 1
            return x % 2 == 0 })
        var count = evens.length()
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "seen").unwrap().value, crate::interpreter::Value::Number(4));
    assert_eq!(Environment::get(&env, "count").unwrap().value, crate::interpreter::Value::Number(2));
}

// Test 71: Comparisons bind more loosely than + and -.
// Regression: both sat on one precedence level, so `1 < 2 + 3` grouped as `(1 < 2) + 3`
// and failed at runtime with "Incompatible types in binary operation".
#[test]
fn test_comparison_binds_looser_than_arithmetic() {
    let code = "
        var a = 1 < 2 + 3
        var b = 1 + 4 > 2
        var c = 10 - 2 <= 8
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 72: `and` binds more tightly than `or`.
// Regression: both sat on one left-associative level, so `true or false and false`
// grouped as `(true or false) and false` and evaluated to false.
// Both cases put `or` first, which is where the two groupings disagree.
#[test]
fn test_and_binds_tighter_than_or() {
    let code = "
        var a = true or false and false
        var b = true or true and false
        var c = 1 < 2 and 3 < 4
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Bool(true));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Bool(true));
}

// Test 73: Loop iterator variables are loop-local.
// Regression: loops inserted the iterator into the enclosing scope, so a surrounding
// binding of the same name was overwritten and left holding the last iteration's value.
#[test]
fn test_loop_variable_does_not_clobber_outer_binding() {
    let code = "
        var i = 7
        var e = \"untouched\"
        loop i from 1..3 {
        }
        loop e in [1, 2] {
        }
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "i").unwrap().value, crate::interpreter::Value::Number(7));
    assert_eq!(Environment::get(&env, "e").unwrap().value, crate::interpreter::Value::Str("untouched".to_string()));
}

// Test 74: The onError variable is handler-local.
// Regression: it was inserted into the enclosing scope, so it overwrote an existing
// binding of the same name and stayed visible after the handler finished.
#[test]
fn test_on_error_variable_is_handler_local() {
    let code = "
        var e = 1
        var r = execute { var z = undefined_thing } onError(e) { \"caught\" }
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "r").unwrap().value, crate::interpreter::Value::Str("caught".to_string()));
    // The handler ran, but `e` still holds the outer value rather than the error message.
    assert_eq!(Environment::get(&env, "e").unwrap().value, crate::interpreter::Value::Number(1));
}

// Test 75: Declarations inside a block stay inside that block.
#[test]
fn test_block_declarations_do_not_leak() {
    let code = "
        if true {
            var inside = 5
        }
        print(inside)
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    let result = Environment::run(&env, &ast);

    assert!(result.is_err(), "block-local declaration should not be visible outside the block");
    assert!(Environment::get(&env, "inside").is_none());
}

// Test 76: Blocks can still read and assign to bindings from enclosing scopes.
#[test]
fn test_blocks_can_mutate_enclosing_bindings() {
    let code = "
        var total = 0
        loop i from 1..4 {
            total = total + i
        }
        var seen = \"\"
        loop x in [1, 2] {
            if true {
                seen = seen + x
            }
        }
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "total").unwrap().value, crate::interpreter::Value::Number(6));
    assert_eq!(Environment::get(&env, "seen").unwrap().value, crate::interpreter::Value::Str("12".to_string()));
}

// Parses and runs one submission against an existing environment, mirroring what the
// REPL's run_code() does: it returns whether the submission succeeded but keeps the
// environment intact either way. Lexer panics are out of scope here (see the lexer
// findings); this covers parse and runtime errors.
fn repl_submit(env: &std::rc::Rc<std::cell::RefCell<Environment>>, code: &str) -> bool {
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(ast) => Environment::run(env, &ast).is_ok(),
        Err(_) => false,
    }
}

// Test 77: REPL state persists across submissions (issue: a fresh Environment per line).
#[test]
fn test_repl_state_persists_across_submissions() {
    let env = Environment::new();

    assert!(repl_submit(&env, "var x = 1"));
    // A later submission can still see `x` and define things in terms of it.
    assert!(repl_submit(&env, "var y = x + 41"));

    assert_eq!(Environment::get(&env, "x").unwrap().value, crate::interpreter::Value::Number(1));
    assert_eq!(Environment::get(&env, "y").unwrap().value, crate::interpreter::Value::Number(42));
}

// Test 78: A failing submission does not end the session or discard prior state
// (issue: run_code() called process::exit on any error).
#[test]
fn test_repl_survives_errors_and_keeps_state() {
    let env = Environment::new();

    assert!(repl_submit(&env, "var kept = 7"));
    // A runtime error: undefined variable.
    assert!(!repl_submit(&env, "print(undefined_var)"));
    // A parse error: malformed statement.
    assert!(!repl_submit(&env, "var = = ="));
    // Prior state is intact and new submissions still work afterwards.
    assert!(repl_submit(&env, "var after = kept + 1"));

    assert_eq!(Environment::get(&env, "kept").unwrap().value, crate::interpreter::Value::Number(7));
    assert_eq!(Environment::get(&env, "after").unwrap().value, crate::interpreter::Value::Number(8));
}

// Test 79: An unknown character is a recoverable lexing error, not a panic.
// The lexer emits Token::Error and the parser surfaces its message.
#[test]
fn test_unknown_character_is_reported_not_panicked() {
    let mut lex = Lexer::new("var x = @");
    let tokens = lex.tokenize();
    // The illegal character becomes an Error token rather than aborting tokenize().
    assert!(tokens.iter().any(|t| matches!(t, Token::Error(_))));

    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown character"));
}

// Test 80: A number literal that does not fit i64 is reported, not panicked.
#[test]
fn test_out_of_range_number_literal_is_reported() {
    let mut lex = Lexer::new("print(99999999999999999999)");
    let tokens = lex.tokenize();
    assert!(tokens.iter().any(|t| matches!(t, Token::Error(_))));

    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of range"));
}

// Test 81: A valid i64 at the boundary still lexes as a Number, not an error.
#[test]
fn test_max_i64_literal_still_lexes() {
    let mut lex = Lexer::new("9223372036854775807");
    let tokens = lex.tokenize();
    assert_eq!(tokens[0], Token::Number(9223372036854775807));
    assert!(!tokens.iter().any(|t| matches!(t, Token::Error(_))));
}

// Test 82: A lone '!' is reported rather than panicking (it is only valid as '!=').
#[test]
fn test_bare_bang_is_reported_not_panicked() {
    let mut lex = Lexer::new("var y = 1\n!");
    let tokens = lex.tokenize();
    assert!(tokens.iter().any(|t| matches!(t, Token::Error(_))));

    let mut parser = Parser::new(tokens);
    assert!(parser.parse_program().is_err());
}

// Test 83: A string with no closing quote is a reported syntax error, not a
// silently-accepted string.
#[test]
fn test_unterminated_string_is_reported() {
    let mut lex = Lexer::new("var x = \"oops");
    let tokens = lex.tokenize();
    // No String token is produced; an Error token is.
    assert!(!tokens.iter().any(|t| matches!(t, Token::String(_))));
    assert!(tokens.iter().any(|t| matches!(t, Token::Error(_))));

    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unterminated string"));
}

// Test 84: A properly closed string still lexes as a single String token,
// including the empty string.
#[test]
fn test_closed_strings_still_lex() {
    let mut lex = Lexer::new("\"hello\"");
    assert_eq!(lex.tokenize()[0], Token::String("hello".to_string()));

    let mut lex_empty = Lexer::new("\"\"");
    assert_eq!(lex_empty.tokenize()[0], Token::String("".to_string()));
}

// Runs a snippet against a fresh environment and returns the result of the run.
fn run_snippet(code: &str) -> Result<(), String> {
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program()?;
    let env = Environment::new();
    Environment::run(&env, &ast)
}

// Test 85: Extension methods called with too few arguments return an interpreter
// error instead of panicking on out-of-bounds indexing into the argument list.
#[test]
fn test_extension_methods_report_missing_arguments() {
    // Each of these previously panicked with an index-out-of-bounds abort.
    for (code, method) in [
        ("var a = []\na.push()", "push"),
        ("print(\"x\".contains())", "contains"),
        ("print(\"x\".replace(\"a\"))", "replace"),
        ("print(\"x\".split())", "split"),
        ("print([].join())", "join"),
        ("print([1].map())", "map"),
        ("print([1].filter())", "filter"),
    ] {
        let result = run_snippet(code);
        assert!(result.is_err(), "expected an error for `{}`", code);
        let msg = result.unwrap_err();
        assert!(
            msg.contains(&format!("{}() expects", method)),
            "unexpected error for `{}`: {}", code, msg
        );
    }
}

// Test 86: The same methods still work when given the right arguments.
#[test]
fn test_extension_methods_still_work_with_arguments() {
    assert!(run_snippet("var a = [1]\na.push(2)").is_ok());
    assert!(run_snippet("print(\"abc\".contains(\"b\"))").is_ok());
    assert!(run_snippet("print(\"aaa\".replace(\"a\", \"b\"))").is_ok());
    assert!(run_snippet("print(\"a,b\".split(\",\").join(\"-\"))").is_ok());
    assert!(run_snippet("print([1, 2].map(fun(x) { return x * 2 }))").is_ok());
    assert!(run_snippet("print([1, 2, 3].filter(fun(x) { return x > 1 }))").is_ok());
}

// Test 87: Integer overflow is a runtime error, not a panic (debug) or a silent
// wraparound (release). Covers every arithmetic operator plus unary negation.
#[test]
fn test_integer_overflow_is_a_runtime_error() {
    let min = "(0 - 9223372036854775807 - 1)"; // i64::MIN, no literal for it
    for code in [
        "print(9223372036854775807 + 1)".to_string(),
        "print(9223372036854775807 * 2)".to_string(),
        format!("print({} - 1)", min),
        format!("print({} / (0 - 1))", min),   // i64::MIN / -1
        format!("var m = {}\nprint(-m)", min), // negate i64::MIN
    ] {
        let result = run_snippet(&code);
        assert!(result.is_err(), "expected overflow error for `{}`", code);
        assert!(result.unwrap_err().contains("integer overflow"));
    }
}

// Test 88a: A `let`-bound container rejects in-place mutation through its own binding,
// both indexed writes and mutating methods; a `var` container still allows both.
#[test]
fn test_let_container_rejects_direct_mutation() {
    // Blocked through the let binding.
    for code in [
        "let xs = [1, 2]\nxs[0] = 9",
        "let xs = [1]\nxs.push(2)",
        "let d = {\"a\": 1}\nd[\"a\"] = 2",
    ] {
        let result = run_snippet(code);
        assert!(result.is_err(), "expected let protection for `{}`", code);
        assert!(result.unwrap_err().contains("'let'"));
    }
    // Allowed for a var container.
    assert!(run_snippet("var xs = [1, 2]\nxs[0] = 9").is_ok());
    assert!(run_snippet("var xs = [1]\nxs.push(2)").is_ok());
    assert!(run_snippet("var d = {\"a\": 1}\nd[\"a\"] = 2").is_ok());
}

// Test 88b: Non-mutating methods on a `let` container are still allowed, since they
// return a new value rather than changing the receiver.
#[test]
fn test_let_container_allows_pure_methods() {
    assert!(run_snippet("let xs = [1, 2]\nprint(xs.map(fun(x) { return x * 2 }))").is_ok());
    assert!(run_snippet("let xs = [1, 2, 3]\nprint(xs.filter(fun(x) { return x > 1 }))").is_ok());
    assert!(run_snippet("let xs = [1, 2, 3]\nprint(xs.length())").is_ok());
    assert!(run_snippet("let xs = [1, 2]\nprint(xs.contains(2))").is_ok());
    assert!(run_snippet("let s = \"hi\"\nprint(s.upper())").is_ok());
}

// Test 88c: An unknown type annotation is rejected whether or not an initializer is
// present, instead of being silently accepted when initialized.
#[test]
fn test_unknown_declared_type_is_rejected() {
    for code in [
        "var x is MadeUp = 1",
        "var x is MadeUp",
        "let y is Nope = 2",
        "var f is Function = fun() { return 1 }",
    ] {
        let result = run_snippet(code);
        assert!(result.is_err(), "expected unknown-type error for `{}`", code);
        assert!(result.unwrap_err().contains("Unknown type"));
    }
}

// Test 88d: Known type annotations still work, and a genuine mismatch is still reported.
#[test]
fn test_known_types_and_mismatches_still_work() {
    // Every known type, with and without an initializer.
    assert!(run_snippet("var a is Number = 5").is_ok());
    assert!(run_snippet("let b is String = \"hi\"").is_ok());
    assert!(run_snippet("var c is Decimal = 3.14").is_ok());
    assert!(run_snippet("var d is Bool = true").is_ok());
    assert!(run_snippet("var e is Array = [1, 2]").is_ok());
    assert!(run_snippet("var f is Dict").is_ok());

    // A real type mismatch is still an error, with the mismatch message (not "Unknown type").
    let mismatch = run_snippet("var x is Number = \"oops\"");
    assert!(mismatch.is_err());
    let msg = mismatch.unwrap_err();
    assert!(msg.contains("Type mismatch"));
    assert!(!msg.contains("Unknown type"));
}

// Test 88: Ordinary integer arithmetic is unchanged, including division and modulo.
#[test]
fn test_integer_arithmetic_still_correct() {
    let env = Environment::new();
    let code = "
        var a = 2 + 3
        var b = 10 * 10
        var c = 7 / 2
        var d = 7 % 3
        var e = -5
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    Environment::run(&env, &ast).unwrap();

    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Number(5));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Number(100));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Number(3));
    assert_eq!(Environment::get(&env, "d").unwrap().value, crate::interpreter::Value::Number(1));
    assert_eq!(Environment::get(&env, "e").unwrap().value, crate::interpreter::Value::Number(-5));
}

// Helper: run a snippet and return the environment so tests can inspect final values.
fn run_env(code: &str) -> std::rc::Rc<std::cell::RefCell<Environment>> {
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    let env = Environment::new();
    Environment::run(&env, &ast).unwrap();
    env
}

// Helper: assert a variable holds an array equal to the given numbers.
fn assert_number_array(env: &std::rc::Rc<std::cell::RefCell<Environment>>, name: &str, expected: &[i64]) {
    let v = Environment::get(env, name).unwrap().value;
    if let crate::interpreter::Value::Array(arr, _) = v {
        let got: Vec<i64> = arr.borrow().iter().map(|e| match e {
            crate::interpreter::Value::Number(n) => *n,
            other => panic!("expected Number, got {:?}", other),
        }).collect();
        assert_eq!(got, expected, "array `{}` mismatch", name);
    } else {
        panic!("`{}` is not an array", name);
    }
}

// Test 89: Assigning a `let` container into a `var` makes an independent copy, so mutating
// the copy leaves the original untouched. This is the alias hole the earlier binding-level
// fix could not close.
#[test]
fn test_var_copy_of_let_is_independent() {
    let env = run_env("
        let xs = [1]
        var ys = xs
        ys.push(99)
    ");
    assert_number_array(&env, "xs", &[1]);
    assert_number_array(&env, "ys", &[1, 99]);
}

// Test 90: The same holds for two `var` containers, and for indexed writes.
#[test]
fn test_var_to_var_assignment_is_independent() {
    let env = run_env("
        var a = [1, 2]
        var b = a
        b[0] = 99
        b.push(3)
    ");
    assert_number_array(&env, "a", &[1, 2]);
    assert_number_array(&env, "b", &[99, 2, 3]);
}

// Test 91: Immutability travels with a container across a function-parameter boundary
// (parameters share by reference): a `let` array cannot be mutated inside the callee.
#[test]
fn test_immutability_travels_into_function() {
    let code = "
        fun f(a) { a.push(7) }
        let xs = [1]
        f(xs)
    ";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    let env = Environment::new();
    let result = Environment::run(&env, &ast);
    assert!(result.is_err(), "mutating a let array inside a function should fail");
    assert!(result.unwrap_err().contains("immutable"));
}

// Test 92: A mutable (var) array passed to a function is still shared and mutated in place
// — the intentional pass-by-reference case.
#[test]
fn test_mutable_container_shares_through_function() {
    let env = run_env("
        fun f(a) { a.push(7) }
        var xs = [1]
        f(xs)
    ");
    assert_number_array(&env, "xs", &[1, 7]);
}

// Test 93: Immutability is deep — a container nested inside a `let` container is frozen too.
#[test]
fn test_deep_immutability() {
    for code in [
        "let m = [[1], [2]]\nm[0].push(9)",
        "let d = {\"a\": [1]}\nd[\"a\"].push(9)",
    ] {
        let mut lex = Lexer::new(code);
        let tokens = lex.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().unwrap();
        let env = Environment::new();
        let result = Environment::run(&env, &ast);
        assert!(result.is_err(), "deep mutation should fail for `{}`", code);
        assert!(result.unwrap_err().contains("immutable"));
    }
}

// Test 94: Embedding an existing container in a new literal copies it, so later changes to
// the original do not leak into the new structure.
#[test]
fn test_literal_embedding_copies_containers() {
    let env = run_env("
        var a = [1]
        var outer = [a]
        a.push(2)
    ");
    // outer holds a copy of a's earlier state, unaffected by the later push.
    let v = Environment::get(&env, "outer").unwrap().value;
    if let crate::interpreter::Value::Array(arr, _) = v {
        assert_eq!(arr.borrow().len(), 1);
        if let crate::interpreter::Value::Array(inner, _) = &arr.borrow()[0] {
            assert_eq!(inner.borrow().len(), 1, "embedded container should be an independent copy");
        } else {
            panic!("outer[0] is not an array");
        }
    } else {
        panic!("outer is not an array");
    }
    assert_number_array(&env, "a", &[1, 2]);
}

// Test 95: Nested indexed assignment now works on a mutable container (a side effect of
// evaluating the assignment target as a value).
#[test]
fn test_nested_index_assignment() {
    let env = run_env("
        var m = [[1, 2], [3, 4]]
        m[0][1] = 99
    ");
    let v = Environment::get(&env, "m").unwrap().value;
    if let crate::interpreter::Value::Array(arr, _) = v {
        if let crate::interpreter::Value::Array(row0, _) = &arr.borrow()[0] {
            let got: Vec<i64> = row0.borrow().iter().map(|e| match e {
                crate::interpreter::Value::Number(n) => *n, _ => panic!(),
            }).collect();
            assert_eq!(got, vec![1, 99]);
        } else { panic!("m[0] not an array"); }
    } else { panic!("m not an array"); }
}

// Test 96: A closure created in a range loop captures that iteration's value of the
// iterator, not the final one. Each iteration runs in its own scope.
#[test]
fn test_range_loop_closures_capture_per_iteration() {
    let env = run_env("
        var fs = []
        loop i from 0..3 { fs.push(fun() { return i }) }
        var a = fs[0]()
        var b = fs[1]()
        var c = fs[2]()
    ");
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Number(0));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Number(1));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Number(2));
}

// Test 97: The same holds for an array-iteration loop.
#[test]
fn test_loop_in_closures_capture_per_iteration() {
    let env = run_env("
        var gs = []
        loop x in [10, 20, 30] { gs.push(fun() { return x }) }
        var a = gs[0]()
        var b = gs[1]()
        var c = gs[2]()
    ");
    assert_eq!(Environment::get(&env, "a").unwrap().value, crate::interpreter::Value::Number(10));
    assert_eq!(Environment::get(&env, "b").unwrap().value, crate::interpreter::Value::Number(20));
    assert_eq!(Environment::get(&env, "c").unwrap().value, crate::interpreter::Value::Number(30));
}

// Test 98: Per-iteration scoping does not break mutation of an enclosing variable from
// inside the loop body.
#[test]
fn test_loop_body_still_mutates_enclosing_variable() {
    let env = run_env("
        var total = 0
        loop i from 1..4 { total = total + i }
        var acc = \"\"
        loop c in [\"a\", \"b\", \"c\"] { acc = acc + c }
    ");
    assert_eq!(Environment::get(&env, "total").unwrap().value, crate::interpreter::Value::Number(6));
    assert_eq!(Environment::get(&env, "acc").unwrap().value, crate::interpreter::Value::Str("abc".to_string()));
}
