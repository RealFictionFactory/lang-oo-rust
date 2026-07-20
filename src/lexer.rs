// src/lexer.rs

/// Represents a single token in the language.
/// Tokens are the smallest units of syntax produced by the Lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Var,
    Let,
    Loop,
    Until,
    In,
    From,
    If,
    Else,
    Is,
    True,
    False,
    And,
    Or,
    Not,
    Func,
    Return,
    Break,
    Continue,
    Use,
    Execute,
    OnError,
    Colon,      // :

    // Literals (values)
    Number(i64),
    Decimal(f64),
    String(String),

    // Identifiers (variable names)
    Ident(String),

    // Operators
    Assign,      // =
    PlusEq,      // +=
    MinusEq,     // -=
    EqEq,        // ==
    NotEq,       // !=
    GreaterThan, // >
    LessThan,    // <
    GreaterEq,   // >=
    LessEq,      // <=
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Range,       // ..

    // Structural characters
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Dot,        // .

    // Special
    NewLine,   // \n - end of line, important when there are no semicolons
    Eof,       // end of file
}

/// The Lexer (or Tokenizer) converts raw source code text into a sequence of Tokens.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    /// Creates a new Lexer instance from a source code string.
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Returns the next character without advancing the position pointer.
    fn peek(&self) -> Option<&char> {
        self.input.get(self.pos)
    }

    /// Returns the next character and advances the position pointer.
    fn next_char(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    /// The main function that converts the input text into a list of tokens.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.next_char() {
            // Ignore spaces and tabs
            if ch.is_whitespace() && ch != '\n' {
                continue;
            }

            // Newline = end of statement in the "Ó" language
            if ch == '\n' {
                tokens.push(Token::NewLine);
                continue;
            }

            // Numbers (integers and floats)
            if ch.is_digit(10) {
                let mut num_str = String::new();
                num_str.push(ch);
                while let Some(&next_ch) = self.peek() {
                    if next_ch.is_digit(10) {
                        num_str.push(next_ch);
                        self.next_char();
                    } else {
                        break;
                    }
                }
                
                // Check for decimal point (but not a range '..')
                if let Some(&'.') = self.peek() {
                    // Look ahead to the next character after '.'
                    if let Some(&next_next) = self.input.get(self.pos + 1) {
                        if next_next.is_digit(10) {
                            // It's a float! (e.g., 3.14)
                            num_str.push('.');
                            self.next_char(); // consume '.'
                            while let Some(&next_ch) = self.peek() {
                                if next_ch.is_digit(10) {
                                    num_str.push(next_ch);
                                    self.next_char();
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Parse as Decimal if it contains a dot, otherwise Number
                if num_str.contains('.') {
                    let num = num_str.parse::<f64>().unwrap();
                    tokens.push(Token::Decimal(num));
                } else {
                    let num = num_str.parse::<i64>().unwrap();
                    tokens.push(Token::Number(num));
                }
                continue;
            }

            // Identifiers and keywords (letters)
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();
                ident.push(ch);
                while let Some(&next_ch) = self.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' { // allow digits and '_' in names
                        ident.push(next_ch);
                        self.next_char();
                    } else {
                        break;
                    }
                }

                // Check if it's a keyword
                let token = match ident.as_str() {
                    "var" => Token::Var,
                    "let" => Token::Let,
                    "loop" | "for" => Token::Loop,
                    "until" => Token::Until,
                    "in" => Token::In,
                    "from" => Token::From,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "is" => Token::Is,
                    "true" => Token::True,
                    "false" => Token::False,
                    "and" => Token::And,
                    "or" => Token::Or,
                    "not" => Token::Not,
                    "func" => Token::Func,
                    "return" => Token::Return,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "use" => Token::Use,
                    "execute" | "exec" | "exe" => Token::Execute,
                    "onError" | "error" => Token::OnError,
                    _ => Token::Ident(ident),
                };
                tokens.push(token);
                continue;
            }

            // Strings (enclosed in double quotes)
            if ch == '"' {
                let mut str_val = String::new();
                while let Some(next_ch) = self.next_char() {
                    if next_ch == '"' {
                        break;
                    }
                    str_val.push(next_ch);
                }
                tokens.push(Token::String(str_val));
                continue;
            }

            // Single-character and two-character operators
            match ch {
                '=' => {
                    if let Some(&'=') = self.peek() {
                        self.next_char();
                        tokens.push(Token::EqEq);
                    } else {
                        tokens.push(Token::Assign);
                    }
                }
                '<' => {
                    if let Some(&'=') = self.peek() {
                        self.next_char();
                        tokens.push(Token::LessEq);
                    } else {
                        tokens.push(Token::LessThan);
                    }
                }
                '>' => {
                    if let Some(&'=') = self.peek() {
                        self.next_char();
                        tokens.push(Token::GreaterEq);
                    } else {
                        tokens.push(Token::GreaterThan);
                    }
                }
                '!' => {
                    if let Some(&'=') = self.peek() {
                        self.next_char();
                        tokens.push(Token::NotEq);
                    } else {
                        panic!("Unknown character: ! (did you mean '!='?)");
                    }
                }
                '+' => {
                    if let Some(&'=') = self.peek() {
                        self.next_char();
                        tokens.push(Token::PlusEq);
                    } else {
                        tokens.push(Token::Plus);
                    }
                }
                '-' => {
                    if let Some(&'=') = self.peek() {
                        self.next_char();
                        tokens.push(Token::MinusEq);
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => tokens.push(Token::Star),
                '/' => {
                    // Check if the next character is also '/'
                    if let Some(&'/') = self.peek() {
                        // We have a comment. Ignore all characters until newline (\n) or EOF.
                        while let Some(&next_ch) = self.peek() {
                            if next_ch == '\n' {
                                break; // End of line - stop ignoring
                            }
                            self.next_char(); // get next character
                        }
                        // Do not add any token here. The main loop will add Token::NewLine
                        // when it reaches '\n' in the next step.
                    } else {
                        // No '/' as the next character - just a division
                        tokens.push(Token::Slash);
                    }
                }
                '%' => tokens.push(Token::Percent),
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),
                '{' => tokens.push(Token::LBrace),
                '}' => tokens.push(Token::RBrace),
                '[' => tokens.push(Token::LBracket),
                ']' => tokens.push(Token::RBracket),
                ',' => tokens.push(Token::Comma),
                '.' => {
                    if let Some(&'.') = self.peek() {
                        self.next_char();
                        tokens.push(Token::Range);
                    } else {
                        tokens.push(Token::Dot);
                    }
                }
                ':' => tokens.push(Token::Colon),
                _ => panic!("Unknown character: {}", ch),
            }
        }

        // Always end the token stream with an End-Of-File marker
        tokens.push(Token::Eof);
        tokens
    }
}
