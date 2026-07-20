// src/stdlib.rs

use crate::interpreter::{Environment, InterpErr, InterpResult, Value, VarInfo};
use std::io::{self, Write};

/// Registers the standard library functions and extensions into the given environment.
/// This includes global functions (like print and input) and extension methods (like asNumber, upper).
pub fn register_stdlib(env: &mut Environment) {
    // Global functions
    env.insert("print".to_string(), VarInfo { 
        value: Value::Builtin(builtin_print), 
        is_const: true 
    });

    env.insert("input".to_string(), VarInfo { 
        value: Value::Builtin(builtin_input), 
        is_const: true 
    });

    // Extension methods
    env.extensions.insert("asNumber".to_string(), ext_as_number);
    env.extensions.insert("asDecimal".to_string(), ext_as_decimal);
    env.extensions.insert("asBoolean".to_string(), ext_as_boolean);
    env.extensions.insert("upper".to_string(), ext_upper);
    env.extensions.insert("lower".to_string(), ext_lower);
    env.extensions.insert("length".to_string(), ext_length);
}

/// print(...args) -> prints values separated by a space, returns Null.
fn builtin_print(args: Vec<Value>) -> InterpResult<Value> {
    let mut parts = Vec::new();
    for arg in &args {
        parts.push(arg.to_string());
    }
    println!("{}", parts.join(" "));
    Ok(Value::Null)
}

/// input("Enter something: ") -> waits for user input, returns String.
fn builtin_input(args: Vec<Value>) -> InterpResult<Value> {
    // Show prompt if provided
    if !args.is_empty() {
        print!("{}", args[0].to_string());
        io::stdout().flush().unwrap();
    }

    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    
    // Remove end-of-line characters
    let input_str = line.trim_end();
    
    Ok(Value::Str(input_str.to_string()))
}

// --- EXTENSION FUNCTIONS ---
// Signature: fn(receiver: Value, args: Vec<Value>) -> InterpResult<Value>

/// Converts a String to an integer (Number). Fails if the string is not a valid integer.
fn ext_as_number(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    if let Value::Str(s) = receiver {
        match s.parse::<i64>() {
            Ok(n) => Ok(Value::Number(n)),
            Err(_) => Err(InterpErr::Err(format!("Cannot convert '{}' to Number", s))),
        }
    } else {
        Err(InterpErr::Err("asNumber() can only be called on String".to_string()))
    }
}

/// Converts a String to a floating-point number (Decimal). Fails if the string is not a valid float.
fn ext_as_decimal(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    if let Value::Str(s) = receiver {
        match s.parse::<f64>() {
            Ok(n) => Ok(Value::Decimal(n)),
            Err(_) => Err(InterpErr::Err(format!("Cannot convert '{}' to Decimal", s))),
        }
    } else {
        Err(InterpErr::Err("asDecimal() can only be called on String".to_string()))
    }
}

/// Converts a String to a Boolean. Recognizes "true"/"1" as true, and "false"/"0" as false.
fn ext_as_boolean(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    if let Value::Str(s) = receiver {
        let lower = s.to_lowercase();
        if lower == "true" || lower == "1" {
            Ok(Value::Bool(true))
        } else if lower == "false" || lower == "0" {
            Ok(Value::Bool(false))
        } else {
            Err(InterpErr::Err(format!("Cannot convert '{}' to Boolean", s)))
        }
    } else {
        Err(InterpErr::Err("asBoolean() can only be called on String".to_string()))
    }
}

/// Returns a new String with all characters converted to uppercase.
fn ext_upper(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    if let Value::Str(s) = receiver {
        Ok(Value::Str(s.to_uppercase()))
    } else {
        Err(InterpErr::Err("upper() can only be called on String".to_string()))
    }
}

/// Returns a new String with all characters converted to lowercase.
fn ext_lower(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    if let Value::Str(s) = receiver {
        Ok(Value::Str(s.to_lowercase()))
    } else {
        Err(InterpErr::Err("lower() can only be called on String".to_string()))
    }
}

// Note that length supports both String and Array!
/// Returns the length of a String (in characters) or an Array (in elements).
fn ext_length(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    match receiver {
        Value::Str(s) => Ok(Value::Number(s.chars().count() as i64)),
        Value::Array(arr) => Ok(Value::Number(arr.len() as i64)),
        _ => Err(InterpErr::Err("length() is only supported on String and Array".to_string())),
    }
}
