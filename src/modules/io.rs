// src/modules/io.rs

use crate::interpreter::{ExtensionFn, InterpErr, InterpResult, Value};
use std::io::Write;
use std::collections::HashMap;

pub fn register(exts: &mut HashMap<String, ExtensionFn>) {
    exts.insert("input".to_string(), ext_input);
}

// Extension function: receiver.input(prompt)
fn ext_input(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpErr::Err("input() expects 1 argument (prompt)".to_string()));
    }
    
    let prompt = args[0].to_string();
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let input_str = line.trim_end();

    // Determine return type based on the receiver's type
    match receiver {
        Value::Number(_) => {
            match input_str.parse::<i64>() {
                Ok(n) => Ok(Value::Number(n)),
                Err(_) => Err(InterpErr::Err(format!("Invalid number input: '{}'", input_str))),
            }
        }
        Value::Decimal(_) => {
            match input_str.parse::<f64>() {
                Ok(n) => Ok(Value::Decimal(n)),
                Err(_) => Err(InterpErr::Err(format!("Invalid decimal input: '{}'", input_str))),
            }
        }
        // Default to String for everything else
        _ => Ok(Value::Str(input_str.to_string())),
    }
}
