// src/main.rs

use std::io::{self, BufRead, Write};
use std::env;
use std::fs;

mod lexer;
mod parser;
mod ast;
mod interpreter;
mod stdlib;
mod modules;

use lexer::Lexer;
use parser::Parser;
use interpreter::Environment;

fn run_code(code: &str) {
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();

    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(ast) => {
            let mut interpreter = Environment::new();
            match interpreter.run(&ast) {
                Ok(_) => {}
                Err(err) => eprintln!("Błąd wykonania: {}", err),
            }
        }
        Err(err) => {
            eprintln!("Błąd składni: {}", err);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Tryb pliku: np. cargo run -- script.oo
    if args.len() > 1 {
        let filename = &args[1];
        
        match fs::read_to_string(filename) {
            Ok(code) => {
                run_code(&code);
            }
            Err(e) => {
                eprintln!("Nie udało się wczytać pliku '{}': {}", filename, e);
            }
        }
    } else {
        // Tryb interaktywny (REPL)
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            let mut input = String::new();
            let mut is_first_line = true;

            loop {
                // Zmieniamy znak zachęty, jeśli jesteśmy w kolejnej linii
                if is_first_line {
                    print!("ó> ");
                } else {
                    print!(".. ");
                }
                stdout.flush().unwrap();

                let mut line = String::new();
                let bytes = stdin.lock().read_line(&mut line).unwrap();
                
                let trimmed = line.trim();

                // Komenda exit kończy program natychmiast
                if trimmed == "exit" {
                    println!("See you around!");
                    return;
                }

                // Kropka w pustej linii kończy czytanie i uruchamia kod
                if trimmed == "." {
                    break;
                }

                // Obsługa Ctrl+D (EOF) - wychodzimy z programu
                if bytes == 0 {
                    if !input.is_empty() {
                        run_code(&input);
                    }
                    println!();
                    return;
                }

                input.push_str(&line);
                is_first_line = false;
            }

            // Jeśli wpisano cokolwiek przed kropką, uruchamiamy
            if !input.trim().is_empty() {
                run_code(&input);
            }
        }
    }
}

// Testy na końcu pliku
#[cfg(test)]
mod tests;
