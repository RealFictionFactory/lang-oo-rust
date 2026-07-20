// src/stdlib.rs

use crate::interpreter::{Environment, InterpResult, Value, VarInfo};

// Registers the standard library functions into the given environment
pub fn register_stdlib(env: &mut Environment) {
    env.insert("print".to_string(), VarInfo { 
        value: Value::Builtin(builtin_print), 
        is_const: true 
    });
}

// print(...args) -> prints values separated by space, returns Null
fn builtin_print(args: Vec<Value>) -> InterpResult<Value> {
    let mut parts = Vec::new();
    for arg in &args {
        parts.push(arg.to_string()); // <--- ZMIEŃ TO
    }
    println!("{}", parts.join(" "));
    Ok(Value::Null)
}
