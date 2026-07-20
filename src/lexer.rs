// src/lexer.rs

// Represents single token in our language
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Var,
    Let,
    Loop,
    From,
    If,
    Else,
    Is,
    True,
    False,
    Func,
    Return,
    Break,
    Continue,
    Use,

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
    NewLine,   // \n - end line, important when no semi colon
    Eof,       // end of file
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    // Zwraca następny znak, nie przesuwając wskaźnika
    fn peek(&self) -> Option<&char> {
        self.input.get(self.pos)
    }

    // Zwraca następny znak, przesuwając wskaźnik
    fn next_char(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    // Główna funkcja zamieniająca tekst na listę tokenów
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.next_char() {
            // Ignorujemy spacje i taby
            if ch.is_whitespace() && ch != '\n' {
                continue;
            }

            // Nowa linia = koniec instrukcji w języku "Ó"
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
                
                // Check for decimal point (but not range '..')
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
                
                // Parse as Decimal if it contains a dot, otherwise Numeric
                if num_str.contains('.') {
                    let num = num_str.parse::<f64>().unwrap();
                    tokens.push(Token::Decimal(num));
                } else {
                    let num = num_str.parse::<i64>().unwrap();
                    tokens.push(Token::Number(num));
                }
                continue;
            }

            // Zmienne i słowa kluczowe (litery)
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();
                ident.push(ch);
                while let Some(&next_ch) = self.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' { // alow digits and  '_' in names
                        ident.push(next_ch);
                        self.next_char();
                    } else {
                        break;
                    }
                }

                // Sprawdzamy, czy to słowo kluczowe
                let token = match ident.as_str() {
                    "var" => Token::Var,
                    "let" => Token::Let,
                    "for" | "loop" => Token::Loop,
                    "from" => Token::From,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "is" => Token::Is,
                    "true" => Token::True,
                    "false" => Token::False,
                    "func" => Token::Func,
                    "return" => Token::Return,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "use" => Token::Use,
                    _ => Token::Ident(ident),
                };
                tokens.push(token);
                continue;
            }

            // Stringi (w podwójnych cudzysłowach)
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

            // Operatory jednoznakowe i dwuznakowe
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
                        panic!("Nieznany znak: ! (czy chodziło o '!='?)");
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
                    // Check if next character is also '/'
                    if let Some(&'/') = self.peek() {
                        // We have a comment. Ignore all characters until (\n)
                        // or EOF.
                        while let Some(&next_ch) = self.peek() {
                            if next_ch == '\n' {
                                break; // End of line - stop ignoring
                            }
                            self.next_char(); // get next character
                        }
                        // Do not add any token here. Main loop will add Token::NewLine
                        // when reaches '\n' in next step
                    } else {
                        // No '/' as a next character - just a division
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
                _ => panic!("Nieznany znak: {}", ch),
            }
        }

        tokens.push(Token::Eof); // Zawsze kończymy strumień tokenów znacznikiem końca pliku
        tokens
    }
}
