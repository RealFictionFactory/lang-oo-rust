// src/parser.rs

use crate::ast::{BinOp, Expr, Stmt};
use crate::lexer::{Token, Lexer};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // Pomocnicze: zwraca obecny token
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    // Pomocnicze: zwraca obecny token i przesuwa wskaźnik
    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    // Pomocnicze: pomija wszystkie tokeny NewLine (ważne przy braku średników!)
    fn skip_newlines(&mut self) {
        while let Some(Token::NewLine) = self.peek() {
            self.next();
        }
    }

    // Główna funkcja parsująca program
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while let Some(token) = self.peek() {
            if token == &Token::Eof {
                break;
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    // Parse single instruction
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Var) => self.parse_var_decl(),
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            Some(Token::Loop) => self.parse_loop(),
            Some(Token::Func) => self.parse_func_decl(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Break) => { self.next(); Ok(Stmt::Break) }
            Some(Token::Continue) => { self.next(); Ok(Stmt::Continue) }
            
            // If it starts with an Ident (like `x = 5` or `print(x)`), it's an assignment or expression statement
            Some(Token::Ident(_)) => {
                let expr = self.parse_expr()?;
                match self.peek().cloned() {
                    Some(Token::Assign) | Some(Token::PlusEq) | Some(Token::MinusEq) => {
                        // It's an assignment
                        self.parse_assign_from_expr(expr)
                    }
                    _ => {
                        // It's just an expression statement (e.g., `print(x)`)
                        Ok(Stmt::ExprStmt(expr))
                    }
                }
            }
            Some(Token::Use) => {
                self.next(); // consume 'use'
                if let Some(Token::Ident(name)) = self.next().cloned() {
                    Ok(Stmt::Use(name))
                } else {
                    Err("Expected module name after 'use'".to_string())
                }
            }
            _ => Err("Unexpected instruction".to_string()),
        }
    }

    // var name [is Type] [= expr]
    fn parse_var_decl(&mut self) -> Result<Stmt, String> {
        self.next(); // consume 'var'
        
        let name = if let Some(Token::Ident(name)) = self.next().cloned() {
            name
        } else {
            return Err("Expected variable name after 'var'".to_string());
        };

        let mut type_name = None;
        if self.peek() == Some(&Token::Is) {
            self.next(); // consume 'is'
            if let Some(Token::Ident(t)) = self.next().cloned() {
                type_name = Some(t);
            } else {
                return Err("Expected type name after 'is'".to_string());
            }
        }

        let mut init_expr = None;
        if self.peek() == Some(&Token::Assign) {
            self.next(); // consume '='
            init_expr = Some(self.parse_expr()?);
        }

        if type_name.is_none() && init_expr.is_none() {
            return Err("Variable declaration must have a type 'is Type' or an initial value '='".to_string());
        }

        Ok(Stmt::VarDecl(name, type_name, init_expr))
    }

    // let name [is Type] [= expr]
    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.next(); // consume 'let'
        
        let name = if let Some(Token::Ident(name)) = self.next().cloned() {
            name
        } else {
            return Err("Expected constant name after 'let'".to_string());
        };

        let mut type_name = None;
        if self.peek() == Some(&Token::Is) {
            self.next(); // consume 'is'
            if let Some(Token::Ident(t)) = self.next().cloned() {
                type_name = Some(t);
            } else {
                return Err("Expected type name after 'is'".to_string());
            }
        }

        let mut init_expr = None;
        if self.peek() == Some(&Token::Assign) {
            self.next(); // consume '='
            init_expr = Some(self.parse_expr()?);
        }

        if type_name.is_none() && init_expr.is_none() {
            return Err("Constant declaration must have a type 'is Type' or an initial value '='".to_string());
        }

        Ok(Stmt::Let(name, type_name, init_expr))
    }

    // if warunek { ... } else { ... }
    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.next(); // zjadamy 'if'
        let condition = self.parse_expr()?;
        
        self.skip_newlines();
        if self.next() != Some(&Token::LBrace) {
            return Err("Oczekiwano '{' po warunku if".to_string());
        }
        
        let if_body = self.parse_block()?;
        
        // --- NOWA LOGIKA DLA ELSE ---
        // Pomijamy nowe linie, bo ktoś może napisać `} \n else`
        self.skip_newlines();
        
        let mut else_body = Vec::new();
        
        // Sprawdzamy, czy po bloku if występuje słowo 'else'
        if self.peek() == Some(&Token::Else) {
            self.next(); // zjadamy 'else'
            
            // Obsługa "else if" - bardzo łatwe w rekurencji!
            // Jeśli po 'else' jest 'if', parsujemy go jako pojedynczą instrukcję w bloku else
            if self.peek() == Some(&Token::If) {
                let nested_if = self.parse_if()?;
                else_body.push(nested_if);
            } else {
                // Zwykły 'else { ... }'
                self.skip_newlines();
                if self.next() != Some(&Token::LBrace) {
                    return Err("Oczekiwano '{' po 'else'".to_string());
                }
                else_body = self.parse_block()?;
            }
        }
        
        Ok(Stmt::If(condition, if_body, else_body))
    }

    // loop i from 1..10 { ... }
    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.next(); // zjadamy 'for'
        
        let var_name = if let Some(Token::Ident(name)) = self.next().cloned() {
            name
        } else {
            return Err("Oczekiwano nazwy zmiennej po 'loop'".to_string());
        };

        if self.next() != Some(&Token::From) {
            return Err("Oczekiwano słowa 'from' w pętli loop".to_string());
        }

        let start_expr = self.parse_expr()?;

        if self.next() != Some(&Token::Range) {
            return Err("Oczekiwano '..' w pętli loop".to_string());
        }

        let end_expr = self.parse_expr()?;

        self.skip_newlines();
        if self.next() != Some(&Token::LBrace) {
            return Err("Oczekiwano '{' po zakresie w loop".to_string());
        }

        let body = self.parse_block()?;
        Ok(Stmt::Loop(var_name, start_expr, end_expr, body))
    }

        // func name(param1, param2) { ... }
    fn parse_func_decl(&mut self) -> Result<Stmt, String> {
        self.next(); // consume 'func'
        
        let name = if let Some(Token::Ident(name)) = self.next().cloned() {
            name
        } else {
            return Err("Expected function name after 'func'".to_string());
        };

        if self.next() != Some(&Token::LParen) {
            return Err("Expected '(' after function name".to_string());
        }

        let mut params = Vec::new();
        if self.peek() != Some(&Token::RParen) {
            loop {
                if let Some(Token::Ident(param)) = self.next().cloned() {
                    params.push(param);
                } else {
                    return Err("Expected parameter name".to_string());
                }
                match self.next() {
                    Some(Token::Comma) => continue,
                    Some(Token::RParen) => break,
                    _ => return Err("Expected ',' or ')' in parameters".to_string()),
                }
            }
        } else {
            self.next(); // consume ')'
        }

        self.skip_newlines();
        if self.next() != Some(&Token::LBrace) {
            return Err("Expected '{' to start function body".to_string());
        }

        let body = self.parse_block()?;
        Ok(Stmt::FuncDecl(name, params, body))
    }

    // return expr
    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.next(); // consume 'return'
        
        // If next token is NewLine, RBrace or Eof, it's an empty return
        if self.peek() == Some(&Token::NewLine) || self.peek() == Some(&Token::RBrace) || self.peek() == Some(&Token::Eof) {
            return Ok(Stmt::Return(None));
        }

        let expr = self.parse_expr()?;
        Ok(Stmt::Return(Some(expr)))
    }

    // Assignment: x = expr, arr[i] = expr, x += expr, etc.
    fn parse_assign_from_expr(&mut self, left: Expr) -> Result<Stmt, String> {
        match self.next().cloned() {
            Some(Token::Assign) => {
                let right = self.parse_expr()?;
                match left {
                    Expr::Variable(name) => Ok(Stmt::Assign(name, right)),
                    Expr::IndexGet(arr_expr, idx_expr) => Ok(Stmt::IndexAssign(arr_expr, idx_expr, right)),
                    _ => Err("Invalid assignment target".to_string()),
                }
            }
            Some(Token::PlusEq) => {
                let right = self.parse_expr()?;
                match left {
                    Expr::Variable(name) => {
                        let left_expr = Expr::Variable(name.clone());
                        let new_expr = Expr::Binary(Box::new(left_expr), BinOp::Add, Box::new(right));
                        Ok(Stmt::Assign(name, new_expr))
                    }
                    Expr::IndexGet(arr_expr, idx_expr) => {
                        let left_clone = Expr::IndexGet(arr_expr.clone(), idx_expr.clone());
                        let new_expr = Expr::Binary(Box::new(left_clone), BinOp::Add, Box::new(right));
                        Ok(Stmt::IndexAssign(arr_expr, idx_expr, new_expr))
                    }
                    _ => Err("Invalid assignment target".to_string()),
                }
            }
            Some(Token::MinusEq) => {
                let right = self.parse_expr()?;
                match left {
                    Expr::Variable(name) => {
                        let left_expr = Expr::Variable(name.clone());
                        let new_expr = Expr::Binary(Box::new(left_expr), BinOp::Subtract, Box::new(right));
                        Ok(Stmt::Assign(name, new_expr))
                    }
                    Expr::IndexGet(arr_expr, idx_expr) => {
                        let left_clone = Expr::IndexGet(arr_expr.clone(), idx_expr.clone());
                        let new_expr = Expr::Binary(Box::new(left_clone), BinOp::Subtract, Box::new(right));
                        Ok(Stmt::IndexAssign(arr_expr, idx_expr, new_expr))
                    }
                    _ => Err("Invalid assignment target".to_string()),
                }
            }
            _ => Err("Expected '=', '+=' or '-=' in assignment".to_string()),
        }
    }

    // Parsuje blok kodu w klamrach { ... }
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while let Some(token) = self.peek() {
            if token == &Token::RBrace {
                self.next(); // zjadamy '}'
                break;
            }
            if token == &Token::Eof {
                return Err("Nieoczekiwany koniec pliku, brak '}'".to_string());
            }
            
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    // --- PARSOWANIE WYRAŻEŃ (EXPRESIONS) ---
    // Na razie uproszczone: zajmuje się tylko + i -
    // Kolejność: parse_expr (dla + i -) -> parse_term (dla * i /) -> parse_factor (liczby, zmienne)
    
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Subtract,
                Token::EqEq => BinOp::Equals,
                Token::NotEq => BinOp::NotEquals,
                Token::GreaterThan => BinOp::GreaterThan,
                Token::LessThan => BinOp::LessThan,
                Token::GreaterEq => BinOp::GreaterEq,
                Token::LessEq => BinOp::LessEq,
                _ => break,
            };

            self.next(); // zjadamy operator
            let right = self.parse_term()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    // Ten poziom obsługuje mnożenie i dzielenie (wyższy priorytet niż + i -)
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Star => BinOp::Multiply,
                Token::Slash => BinOp::Divide,
                Token::Percent => BinOp::Modulo,
                _ => break,
            };
            self.next(); // zjadamy operator
            
            let right = self.parse_factor()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    // Najniższy poziom: liczby, stringi, zmienne, (wyrazenie)
    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = match self.next().cloned() {
            Some(Token::Number(n)) => Expr::Number(n),
            
            Some(Token::Decimal(n)) => Expr::Decimal(n),

            // String interpolation: "text {expr} text"
            Some(Token::String(s)) => {
                if !s.contains('{') {
                    Expr::Str(s)
                } else {
                    let mut parts: Vec<Expr> = Vec::new();
                    let mut chars = s.chars().peekable();
                    let mut current_part = String::new();

                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            // Save the preceding text as a string expression
                            if !current_part.is_empty() {
                                parts.push(Expr::Str(current_part.clone()));
                                current_part.clear();
                            }

                            // Extract the string inside { }
                            let mut inner_str = String::new();
                            while let Some(inner_ch) = chars.next() {
                                if inner_ch == '}' {
                                    break;
                                }
                                inner_str.push(inner_ch);
                            }

                            // Create a new Lexer and Parser just for the inside of { }!
                            let mut inner_lex = Lexer::new(&inner_str);
                            let inner_tokens = inner_lex.tokenize();
                            let mut inner_parser = Parser::new(inner_tokens);
                            let inner_expr = inner_parser.parse_expr()?;
                            parts.push(inner_expr);
                        } else {
                            current_part.push(ch);
                        }
                    }

                    // Save any remaining text after the last { }
                    if !current_part.is_empty() {
                        parts.push(Expr::Str(current_part));
                    }

                    // Combine all parts using Binary Add (e.g., "text" + expr + "text")
                    let mut iter = parts.into_iter();
                    let mut expr = iter.next().unwrap_or(Expr::Str(String::new()));
                    for next_expr in iter {
                        expr = Expr::Binary(Box::new(expr), BinOp::Add, Box::new(next_expr));
                    }
                    expr
                }
            }

            Some(Token::True) => Expr::Bool(true),

            Some(Token::False) => Expr::Bool(false),

            Some(Token::Ident(name)) => Expr::Variable(name),

            Some(Token::LParen) => {
                let e = self.parse_expr()?;
                if self.next() != Some(&Token::RParen) {
                    return Err("Expected ')'".to_string());
                }
                e
            }

            // Array literal: [1, 2, 3]
            Some(Token::LBracket) => {
                let mut elements = Vec::new();
                
                // Handle empty array []
                if self.peek() == Some(&Token::RBracket) {
                    self.next(); // consume ']'
                } else {
                    loop {
                        let e = self.parse_expr()?;
                        elements.push(e);
                        match self.next() {
                            Some(Token::Comma) => continue,
                            Some(Token::RBracket) => break,
                            _ => return Err("Expected ',' or ']' in array literal".to_string()),
                        }
                    }
                }
                Expr::Array(elements)
            }
            _ => return Err("Unexpected token in expression".to_string()),
        };

        // Postfix indexing: arr[0] or arr[i]
        // We use a while loop to allow multidimensional access like matrix[0][1]
        while self.peek() == Some(&Token::LBracket) {
            self.next(); // consume '['
            let index = self.parse_expr()?;
            if self.next() != Some(&Token::RBracket) {
                return Err("Expected ']' after index".to_string());
            }
            expr = Expr::IndexGet(Box::new(expr), Box::new(index));
        }

        // Postfix method call: obj.method(args)
        while self.peek() == Some(&Token::Dot) {
            self.next(); // consume '.'
            
            let method_name = if let Some(Token::Ident(name)) = self.next().cloned() {
                name
            } else {
                return Err("Expected method name after '.'".to_string());
            };

            if self.next() != Some(&Token::LParen) {
                return Err("Expected '(' after method name".to_string());
            }

            let mut args = Vec::new();
            if self.peek() != Some(&Token::RParen) {
                loop {
                    let arg = self.parse_expr()?;
                    args.push(arg);
                    match self.next() {
                        Some(Token::Comma) => continue,
                        Some(Token::RParen) => break,
                        _ => return Err("Expected ',' or ')' in method arguments".to_string()),
                    }
                }
            } else {
                self.next(); // consume ')'
            }

            expr = Expr::MethodCall(Box::new(expr), method_name, args);
        }

        // Postfix function call: foo(args)
        while self.peek() == Some(&Token::LParen) {
            self.next(); // consume '('
            let mut args = Vec::new();
            
            if self.peek() != Some(&Token::RParen) {
                loop {
                    let arg = self.parse_expr()?;
                    args.push(arg);
                    match self.next() {
                        Some(Token::Comma) => continue,
                        Some(Token::RParen) => break,
                        _ => return Err("Expected ',' or ')' in function call".to_string()),
                    }
                }
            } else {
                self.next(); // consume ')'
            }
            
            expr = Expr::Call(Box::new(expr), args);
        }

        Ok(expr)
    }
}
