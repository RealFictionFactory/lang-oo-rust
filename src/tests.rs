// src/tests.rs

use crate::lexer::{Lexer, Token};
use crate::parser::Parser;
use crate::interpreter::Environment;
use crate::ast::Stmt;

// Test 1: Sprawdzamy, czy lekser poprawnie dzieli kod na tokeny
#[test]
fn test_lexer_basic() {
    let mut lex = Lexer::new("var x = 10");
    let tokens = lex.tokenize();

    assert_eq!(tokens[0], Token::Var);
    assert_eq!(tokens[1], Token::Ident("x".to_string()));
    assert_eq!(tokens[2], Token::Assign);
    assert_eq!(tokens[3], Token::Number(10));
}

// Test 2: Sprawdzamy, czy komentarze są poprawnie ignorowane przez lekser
#[test]
fn test_lexer_comments() {
    let mut lex = Lexer::new("var x = 5 // to jest komentarz\n");
    let tokens = lex.tokenize();
    
    // Oczekujemy, że po liczbie 5 od razu pojawi się nowa linia, a komentarz zniknie
    assert_eq!(tokens[3], Token::Number(5));
    assert_eq!(tokens[4], Token::NewLine);
}

// Test 3: Sprawdzamy, czy parser poprawnie buduje drzewo dla deklaracji zmiennej
#[test]
fn test_parser_var_decl() {
    let mut lex = Lexer::new("var y = 20");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap(); // unwrap() jest ok w testach, jak wywali błąd - test nie przejdzie

    assert_eq!(ast.len(), 1); // Powinna być jedna instrukcja
    // Możemy też sprawdzić, czy to właściwy węzeł (wymaga importu Stmt z ast)
    // assert_eq!(ast[0], crate::ast::Stmt::VarDecl("y".to_string(), crate::ast::Expr::Number(20)));
}

// Test 4: Sprawdzamy, czy interpreter nie wyrzuca błędów przy prostym kodzie
#[test]
fn test_interpreter_runs_without_error() {
    let code = "var a = 10\nvar b = 20\nprint(a + b)";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok()); // Sprawdzamy, czy wykonanie zakończyło się sukcesem
}

// Test 5: Sprawdzamy błędy składniowe (np. brak '=' i 'is' po nazwie zmiennej)
#[test]
fn test_parser_syntax_error() {
    let mut lex = Lexer::new("var x 10");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    
    assert!(result.is_err()); // Oczekujemy, że parser zwróci błąd (Err)
    assert_eq!(result.unwrap_err(), "Variable declaration must have a type 'is Type' or an initial value '='");
}

// Test 6: Sprawdzamy, czy parser poprawnie buduje AST z 'if' i 'else'
#[test]
fn test_parser_if_else_ast() {
    let code = "if x == 5 { print(1) } else { print(2) }";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        Stmt::If(_, if_body, else_body) => {
            // Sprawdzamy, czy oba bloki mają po jednej instrukcji (print)
            assert_eq!(if_body.len(), 1, "Blok 'if' powinien mieć 1 instrukcję");
            assert_eq!(else_body.len(), 1, "Blok 'else' powinien mieć 1 instrukcję");
        }
        _ => panic!("Oczekiwano węzła Stmt::If, otrzymano coś innego!"),
    }
}

// Test 7: Sprawdzamy poprawne wykonanie 'if/else' przez interpreter
// Zwróć uwagę, że kod w teście nie ma 'print', po prostu sprawdzamy czy 
// wykonanie kodu, który wchodzi w 'else' nie powoduje błędów.
#[test]
fn test_interpreter_if_else_execution() {
    // x wynosi 10, więc warunek x == 5 jest fałszywy. Powinien wejść w 'else'.
    let code = "var x = 10\nif x == 5 { var a = 1 } else { var b = 2 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok(), "Interpreter powinien wykonać kod bez błędów");
}

// Test 8: Sprawdzamy, czy pętla 'loop' poprawnie się buduje
#[test]
fn test_parser_for_loop_ast() {
    let code = "loop i from 1..5 { print(i) }";
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        // Wzorzec: Loop(nazwa_zmiennej, start, end, blok_kodu)
        Stmt::Loop(var_name, _, _, body) => {
            assert_eq!(var_name, "i", "Zmienna iteracyjna powinna nazywać się 'i'");
            assert_eq!(body.len(), 1, "Blok pętli powinien mieć 1 instrukcję");
        }
        _ => panic!("Oczekiwano węzła Stmt::Loop, otrzymano coś innego!"),
    }
}

// Test 9: Sprawdzamy, czy interpreter używa truthiness (0 jest false, 5 jest true)
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

// Test 10: Sprawdzamy wykonanie pętli 'for' z użyciem zmiennej iteracyjnej
#[test]
fn test_interpreter_for_loop_runs() {
    // Pętla, która po prostu wykonuje operację matematyczną. 
    // Nie wypisujemy na ekran, żeby nie śmiecić w konsoli podczas testów.
    let code = "loop i from 1..3 { var x = i + 10 }";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok(), "Pętla powinna wykonać się bez błędów");
}

// Test 11: Sprawdzamy, czy można nadpisać zmienną (Assign)
#[test]
fn test_interpreter_variable_assignment() {
    let code = "var x = 5\nx = 20";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_ok(), "Przypisanie do istniejącej zmiennej powinno działać");
    
    // Wchodzimy do VarInfo i sprawdzamy pole .value
    let var_info = env.get("x").expect("Zmienna x powinna istnieć");
    assert_eq!(var_info.value, crate::interpreter::Value::Number(20));
    assert!(!var_info.is_const, "x nie powinno być stałą");
}

// Test 12: Sprawdzamy, czy przypisanie do niezadeklarowanej zmiennej rzuca błąd
#[test]
fn test_interpreter_assign_undeclared_fails() {
    let code = "y = 10";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_err(), "Powinien wystąpić błąd przypisania do niezadeklarowanej zmiennej");
    assert_eq!(
        result.unwrap_err(), 
        "Zmienna 'y' nie jest zadeklarowana. Użyj 'var' lub 'let'."
    );
}

// Test 13: Sprawdzamy, czy próba zmiany stałej (let) kończy się błędem
#[test]
fn test_interpreter_const_reassignment_fails() {
    let code = "let y = 10\ny = 20";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_err(), "Powinien wystąpić błąd przy zmianie stałej");
    assert_eq!(
        result.unwrap_err(), 
        "Nie można zmienić wartości stałej 'y'"
    );
}

// Test 14: Sprawdzamy, czy parser poprawnie buduje AST dla stałej 'let'
#[test]
fn test_parser_let_decl() {
    let mut lex = Lexer::new("let y = 20");
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    assert_eq!(ast.len(), 1);
    // Sprawdzamy, czy to węzeł Let
    assert!(matches!(ast[0], Stmt::Let(..)), "Oczekiwano węzła Stmt::Let");
}

// Test 15: Sprawdzamy priorytet operatorów (mnożenie przed dodawaniem)
#[test]
fn test_interpreter_math_precedence() {
    // 2 + 3 * 4 powinno dać 14, a nie 20
    let code = "var x = 2 + 3 * 4";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    let var_info = env.get("x").expect("Zmienna x powinna istnieć");
    assert_eq!(var_info.value, crate::interpreter::Value::Number(14));
}

// Test 16: Sprawdzamy, czy dzielenie przez zero rzuca błąd
#[test]
fn test_interpreter_division_by_zero() {
    let code = "var y = 10 / 0";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    let result = env.run(&ast);
    
    assert!(result.is_err(), "Dzielenie przez zero powinno rzucić błąd");
    assert_eq!(result.unwrap_err(), "Błąd wykonania: Dzielenie przez zero!");
}

// Test 17: Sprawdzamy operatory porównania
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

// Test 18: Sprawdzamy skrócone operatory przypisania (+=, -=)
#[test]
fn test_interpreter_compound_assignment() {
    // var x = 5
    // x += 10  (x powinno wynosić 15)
    // x -= 3   (x powinno wynosić 12)
    let code = "var x = 5\nx += 10\nx -= 3";
    
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    
    let mut env = Environment::new();
    env.run(&ast).unwrap();
    
    let var_info = env.get("x").expect("Zmienna x powinna istnieć");
    assert_eq!(var_info.value, crate::interpreter::Value::Number(12));
}

// Test 19: Sprawdzamy obsługę literałów logicznych (true/false)
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
