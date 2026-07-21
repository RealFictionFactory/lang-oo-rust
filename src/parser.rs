// src/parser.rs

use crate::ast::{BinOp, Expr, Stmt, UnOp};
use crate::lexer::{Token, Lexer};

/// The Parser takes a list of tokens from the Lexer and constructs an Abstract Syntax Tree (AST).
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Creates a new Parser instance from a vector of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Helper: returns the current token without advancing the position pointer.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Helper: returns the current token and advances the position pointer.
    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// Helper: skips all NewLine tokens (important since the language does not use semicolons).
    fn skip_newlines(&mut self) {
        while let Some(Token::NewLine) = self.peek() {
            self.next();
        }
    }

    /// Main function that parses the entire program into a list of statements.
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

    /// Parses a single statement/instruction.
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Var) => self.parse_var_decl(),
            
            Some(Token::Let) => self.parse_let(),
            
            Some(Token::Loop) => self.parse_loop(),
            
            Some(Token::Func) => self.parse_func_decl(),
            
            Some(Token::Return) => self.parse_return(),
            
            Some(Token::Break) => { self.next(); Ok(Stmt::Break) }
            
            Some(Token::Continue) => { self.next(); Ok(Stmt::Continue) }
            
            Some(Token::Until) => {
                self.next(); // consume 'until'
                // Oczekujemy nawiasów: until (warunek)
                if self.next() != Some(&Token::LParen) {
                    return Err("Expected '(' after 'until'".to_string());
                }
                let condition = self.parse_expr()?;
                if self.next() != Some(&Token::RParen) {
                    return Err("Expected ')' after until condition".to_string());
                }
                Ok(Stmt::Until(condition))
            }

            Some(Token::Use) => {
                self.next(); // consume 'use'
                if let Some(Token::Ident(name)) = self.next().cloned() {
                    Ok(Stmt::Use(name))
                } else {
                    Err("Expected module name after 'use'".to_string())
                }
            }

            // Fallback for assignments and expression statements.
            // Catches anything else (Ident, Number, execute, etc.)
            _ => {
                let expr = self.parse_expr()?;
                match self.peek().cloned() {
                    Some(Token::Assign) | Some(Token::PlusEq) | Some(Token::MinusEq) => {
                        self.parse_assign_from_expr(expr)
                    }
                    _ => {
                        Ok(Stmt::ExprStmt(expr))
                    }
                }
            }
        }
    }

    /// Parses a mutable variable declaration: `var name [is Type] [= expr]`
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

    /// Parses an immutable constant declaration: `let name [is Type] [= expr]`
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

    /// Parses all loop variants: `loop i from..`, `loop x in arr`, `loop { ... }`
    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.next(); // consume 'loop'
        
        // Wariant 1: loop { ... } (pętla nieskończona/do-while)
        if self.peek() == Some(&Token::LBrace) {
            self.next(); // consume '{'
            let body = self.parse_block()?;
            return Ok(Stmt::LoopBlock(body));
        }
        
        // Zmienna iteracyjna (dla from i dla in)
        let var_name = if let Some(Token::Ident(name)) = self.next().cloned() {
            name
        } else {
            return Err("Expected variable name after 'loop'".to_string());
        };

        // Wariant 2: loop element in array { ... }
        if self.peek() == Some(&Token::In) {
            self.next(); // consume 'in'
            let iterable_expr = self.parse_expr()?;
            
            self.skip_newlines();
            if self.next() != Some(&Token::LBrace) {
                return Err("Expected '{' after array in loop".to_string());
            }
            let body = self.parse_block()?;
            return Ok(Stmt::LoopIn(var_name, iterable_expr, body));
        }

        // Wariant 3: loop i from start..end { ... } (obecna logika)
        if self.next() != Some(&Token::From) {
            return Err("Expected 'from', 'in' or '{' in loop".to_string());
        }

        let start_expr = self.parse_expr()?;
        if self.next() != Some(&Token::Range) {
            return Err("Expected '..' in loop".to_string());
        }
        let end_expr = self.parse_expr()?;

        self.skip_newlines();
        if self.next() != Some(&Token::LBrace) {
            return Err("Expected '{' after range in loop".to_string());
        }

        let body = self.parse_block()?;
        Ok(Stmt::Loop(var_name, start_expr, end_expr, body))
    }

    /// Parses a function declaration: `func name(param1, param2) { ... }`
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

    /// Parses a return statement: `return [expr]`
    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.next(); // consume 'return'
        
        // If next token is NewLine, RBrace or Eof, it's an empty return
        if self.peek() == Some(&Token::NewLine) || self.peek() == Some(&Token::RBrace) || self.peek() == Some(&Token::Eof) {
            return Ok(Stmt::Return(None));
        }

        let expr = self.parse_expr()?;
        Ok(Stmt::Return(Some(expr)))
    }

    /// Parses an assignment: `x = expr`, `arr[i] = expr`, `x += expr`, etc.
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

    /// Parses a block of code enclosed in braces `{ ... }`
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while let Some(token) = self.peek() {
            if token == &Token::RBrace {
                self.next(); // consume '}'
                break;
            }
            if token == &Token::Eof {
                return Err("Unexpected end of file, missing '}'".to_string());
            }
            
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    // --- EXPRESSION PARSING ---
    // Precedence: parse_expr (+, -, ==, !=, >, <) -> parse_term (*, /, %) -> parse_factor (literals, variables, parentheses)
    
    /// Parses expressions handling logical operators (and, or).
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logic()?;

        while let Some(token) = self.peek() {
            let op = match token {
                Token::And => BinOp::And,
                Token::Or => BinOp::Or,
                _ => break,
            };

            self.next(); // consume operator
            let right = self.parse_logic()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// Parses logical operations (and, or) by calling the arithmetic parser.
    fn parse_logic(&mut self) -> Result<Expr, String> {
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

            self.next(); // consume operator
            let right = self.parse_term()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// Parses expressions handling multiplication, division, and modulo (higher precedence than + and -).
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Star => BinOp::Multiply,
                Token::Slash => BinOp::Divide,
                Token::Percent => BinOp::Modulo,
                _ => break,
            };
            self.next(); // consume operator
            
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// Parses unary operators (- and not) before the main factor.
    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Minus) => {
                self.next(); // consume '-'
                let right = self.parse_unary()?;
                Ok(Expr::Unary(UnOp::Negate, Box::new(right)))
            }
            Some(Token::Not) => {
                self.next(); // consume 'not'
                let right = self.parse_unary()?;
                Ok(Expr::Unary(UnOp::Not, Box::new(right)))
            }
            _ => self.parse_factor(),
        }
    }

    /// Parses the lowest level expressions: literals, variables, parentheses, arrays, and string interpolation.
    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = match self.next().cloned() {
            Some(Token::Match) => {
                let condition = self.parse_expr()?;
                
                self.skip_newlines();
                if self.next() != Some(&Token::LBrace) {
                    return Err("Expected '{' after match expression".to_string());
                }
                
                let mut arms = Vec::new();
                self.skip_newlines();
                
                while self.peek() != Some(&Token::RBrace) {
                    // Pattern: either '_' (wildcard) or an expression
                    let pattern = if self.peek() == Some(&Token::Ident("_".to_string())) {
                        self.next(); // consume '_'
                        None
                    } else {
                        Some(self.parse_expr()?)
                    };
                    
                    if self.next() != Some(&Token::Arrow) {
                        return Err("Expected '->' in match arm".to_string());
                    }
                    
                    // Body: either a block { ... } or a single expression
                    let body = if self.peek() == Some(&Token::LBrace) {
                        self.next(); // consume '{'
                        self.parse_block()?
                    } else {
                        let expr = self.parse_expr()?;
                        vec![Stmt::ExprStmt(expr)]
                    };
                    
                    arms.push((pattern, body));
                    self.skip_newlines();
                }
                
                self.next(); // consume '}'
                return Ok(Expr::Match(Box::new(condition), arms));
            }

            Some(Token::If) => {
                let condition = self.parse_expr()?;
                
                self.skip_newlines();
                if self.next() != Some(&Token::LBrace) {
                    return Err("Expected '{' after if condition".to_string());
                }
                let if_body = self.parse_block()?;
                
                // --- LOGIC FOR ELSE ---
                self.skip_newlines();
                let mut else_body = Vec::new();
                
                if self.peek() == Some(&Token::Else) {
                    self.next(); // consume 'else'
                    self.skip_newlines();
                    
                    if self.peek() == Some(&Token::If) {
                        // 'else if' - rekurencja! Sparsuj kolejnego ifa jako wyrażenie
                        let nested_if_expr = self.parse_expr()?;
                        else_body.push(Stmt::ExprStmt(nested_if_expr));
                    } else {
                        if self.next() != Some(&Token::LBrace) {
                            return Err("Expected '{' after 'else'".to_string());
                        }
                        else_body = self.parse_block()?;
                    }
                }
                
                return Ok(Expr::If(Box::new(condition), if_body, else_body));
            }

            Some(Token::Execute) => {
                self.skip_newlines();
                if self.next() != Some(&Token::LBrace) {
                    return Err("Expected '{' after 'execute'".to_string());
                }
                let run_body = self.parse_block()?;
                
                self.skip_newlines();
                
                // 'onError' is required for this expression
                if self.next() != Some(&Token::OnError) {
                    return Err("Expected 'onError' after 'execute' block".to_string());
                }
                
                let mut err_var = None;
                
                // Optional (err) parameter
                self.skip_newlines();
                if self.peek() == Some(&Token::LParen) {
                    self.next(); // consume '('
                    if let Some(Token::Ident(name)) = self.next().cloned() {
                        err_var = Some(name);
                    } else {
                        return Err("Expected error variable name in onError()".to_string());
                    }
                    if self.next() != Some(&Token::RParen) {
                        return Err("Expected ')' after error variable name".to_string());
                    }
                }
                
                self.skip_newlines();
                if self.next() != Some(&Token::LBrace) {
                    return Err("Expected '{' after 'onError'".to_string());
                }
                let catch_body = self.parse_block()?;
                
                return Ok(Expr::ExecuteCatch(run_body, err_var, catch_body));
            }

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

            // Dictionary literal: {"key": value, ...}
            Some(Token::LBrace) => {
                let mut pairs = Vec::new();
                
                // Handle empty dict {}
                if self.peek() == Some(&Token::RBrace) {
                    self.next(); // consume '}'
                } else {
                    loop {
                        // Key must be an expression (usually a String literal)
                        let key = self.parse_expr()?;
                        
                        if self.next() != Some(&Token::Colon) {
                            return Err("Expected ':' after dictionary key".to_string());
                        }
                        
                        let value = self.parse_expr()?;
                        pairs.push((key, value));
                        
                        match self.next() {
                            Some(Token::Comma) => continue,
                            Some(Token::RBrace) => break,
                            _ => return Err("Expected ',' or '}' in dictionary literal".to_string()),
                        }
                    }
                }
                Expr::Dict(pairs)
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

        // Postfix parsing loop
        loop {
            match self.peek() {
                // Postfix indexing: arr[0] or arr[i]
                // Allows multidimensional access like matrix[0][1]
                Some(Token::LBracket) => {
                    self.next(); // consume '['
                    let index = self.parse_expr()?;
                    if self.next() != Some(&Token::RBracket) {
                        return Err("Expected ']' after index".to_string());
                    }
                    expr = Expr::IndexGet(Box::new(expr), Box::new(index));
                }
                
                // Postfix method call: obj.method(args)
                Some(Token::Dot) => {
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
                Some(Token::LParen) => {
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
                
                // If it's not '[', '.' or '(', we stop parsing postfixes
                _ => break,
            }
        }

        Ok(expr)
    }
}
