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

/// Executes the provided source code string.
/// This function handles the entire pipeline: Lexing -> Parsing -> Evaluation.
/// Any syntax or runtime errors are printed to standard error.
fn run_code(code: &str) {
    let mut lex = Lexer::new(code);
    let tokens = lex.tokenize();

    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(ast) => {
            let interpreter = Environment::new();
            match Environment::run(&interpreter, &ast) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Runtime error: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Syntax error: {}", err);
            std::process::exit(1);
        }
    }
}

/// Entry point of the program.
/// Supports two modes:
/// 1. File mode: runs a script passed as a command-line argument (e.g., `cargo run -- script.oo`).
/// 2. Interactive mode (REPL): allows typing and evaluating code line-by-line.
fn main() {
    let args: Vec<String> = env::args().collect();

    // File mode: e.g., cargo run -- script.oo
    if args.len() > 1 {
        let filename = &args[1];
        
        match fs::read_to_string(filename) {
            Ok(code) => {
                run_code(&code);
            }
            Err(e) => {
                eprintln!("Failed to read file '{}': {}", filename, e);
            }
        }
    } else {
        // Interactive mode (REPL)
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            let mut input = String::new();
            let mut is_first_line = true;

            loop {
                // Change the prompt if we are on a subsequent line
                if is_first_line {
                    print!("ó> ");
                } else {
                    print!(".. ");
                }
                stdout.flush().unwrap();

                let mut line = String::new();
                let bytes = stdin.lock().read_line(&mut line).unwrap();
                
                let trimmed = line.trim();

                // The 'exit' command terminates the program immediately
                if trimmed == "exit" {
                    println!("See you around!");
                    return;
                }

                // A dot on an empty line ends input and runs the code
                if trimmed == "." {
                    break;
                }

                // Handle Ctrl+D (EOF) - exit the program
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

            // If anything was entered before the dot, run it
            if !input.trim().is_empty() {
                run_code(&input);
            }
        }
    }
}

// Tests at the end of the file
#[cfg(test)]
mod tests;
