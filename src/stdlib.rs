// src/stdlib.rs

use crate::interpreter::{Environment, InterpErr, InterpResult, Value, VarInfo};
use std::cell::RefCell;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::rc::Rc;

/// Registers the standard library functions and extensions into the given environment.
/// This includes global functions (like print and input) and extension methods (like asNumber, upper).
pub fn register_stdlib(env: &Rc<RefCell<Environment>>) {
    // Global functions
    Environment::insert(env, "print".to_string(), VarInfo { 
        value: Value::Builtin(builtin_print), 
        is_const: true,
        type_name: None
    });

    Environment::insert(env, "input".to_string(), VarInfo { 
        value: Value::Builtin(builtin_input), 
        is_const: true,
        type_name: None
    });

    Environment::insert(env, "args".to_string(), VarInfo { 
        value: Value::Builtin(builtin_args), 
        is_const: true,
        type_name: None
    });

    Environment::insert(env, "exit".to_string(), VarInfo { 
        value: Value::Builtin(builtin_exit), 
        is_const: true,
        type_name: None
    });

    Environment::insert(env, "shell".to_string(), VarInfo { 
        value: Value::Builtin(builtin_shell), 
        is_const: true,
        type_name: None
    });

    // Extension methods
    let mut env_mut = env.borrow_mut();
    env_mut.extensions.insert("asNumber".to_string(), ext_as_number);
    env_mut.extensions.insert("asDecimal".to_string(), ext_as_decimal);
    env_mut.extensions.insert("asBoolean".to_string(), ext_as_boolean);
    env_mut.extensions.insert("upper".to_string(), ext_upper);
    env_mut.extensions.insert("lower".to_string(), ext_lower);
    env_mut.extensions.insert("length".to_string(), ext_length);
    env_mut.extensions.insert("push".to_string(), ext_push);
    env_mut.extensions.insert("trim".to_string(), ext_trim);
    env_mut.extensions.insert("contains".to_string(), ext_contains);
    env_mut.extensions.insert("replace".to_string(), ext_replace);
    env_mut.extensions.insert("split".to_string(), ext_split);
    env_mut.extensions.insert("contains".to_string(), ext_contains);
    env_mut.extensions.insert("join".to_string(), ext_join);
    env_mut.extensions.insert("map".to_string(), ext_map);
    env_mut.extensions.insert("filter".to_string(), ext_filter);
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

/// Returns an interpreter error unless at least `expected` arguments were supplied.
/// Extension methods call this before indexing into `args`, so a call like `[].push()`
/// produces a language-level error instead of panicking on out-of-bounds indexing.
fn check_arity(method: &str, args: &[Value], expected: usize) -> InterpResult<()> {
    if args.len() < expected {
        return Err(InterpErr::Err(format!(
            "{}() expects {} argument(s), got {}",
            method, expected, args.len()
        )));
    }
    Ok(())
}

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
        Value::Array(arr, _) => Ok(Value::Number(arr.borrow().len() as i64)),
        _ => Err(InterpErr::Err("length() is only supported on String and Array".to_string())),
    }
}

/// Adds an element to the end of the array (mutates the array in place via Rc<RefCell>).
/// Refused on an immutable array; the pushed element is stored as an independent, mutable
/// copy (value semantics), matching how indexed assignment stores its value.
fn ext_push(receiver: Value, mut args: Vec<Value>) -> InterpResult<Value> {
    check_arity("push", &args, 1)?;
    if let Value::Array(arr, immutable) = receiver {
        if immutable.get() {
            return Err(InterpErr::Err("Cannot push to an immutable array (declared with 'let')".to_string()));
        }
        let element = Environment::deep_bind(args.remove(0), false);
        arr.borrow_mut().push(element);
        Ok(Value::Null)
    } else {
        Err(InterpErr::Err("push() can only be called on Array".to_string()))
    }
}

/// Returns a new String with leading and trailing whitespace removed.
fn ext_trim(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    if let Value::Str(s) = receiver {
        Ok(Value::Str(s.trim().to_string()))
    } else {
        Err(InterpErr::Err("trim() can only be called on String".to_string()))
    }
}

/// Returns true if the String contains the given substring, or if the Array contains the given element.
fn ext_contains(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    check_arity("contains", &args, 1)?;
    match receiver {
        Value::Str(s) => {
            if let Value::Str(sub) = &args[0] {
                Ok(Value::Bool(s.contains(sub)))
            } else {
                Err(InterpErr::Err("contains() on String requires a String argument".to_string()))
            }
        }
        Value::Array(arr, _) => {
            let target = &args[0];
            let arr_ref = arr.borrow();
            // Use our custom PartialEq to find the element
            Ok(Value::Bool(arr_ref.iter().any(|v| v == target)))
        }
        _ => Err(InterpErr::Err("contains() is only supported on String and Array".to_string()))
    }
}

/// Returns a new String where all occurrences of `old` are replaced with `new`.
fn ext_replace(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    check_arity("replace", &args, 2)?;
    if let (Value::Str(s), Value::Str(old), Value::Str(new)) = (receiver, &args[0], &args[1]) {
        Ok(Value::Str(s.replace(old, new)))
    } else {
        Err(InterpErr::Err("replace() requires a String receiver and two String arguments".to_string()))
    }
}

/// Splits the String by a separator and returns an Array of Strings.
fn ext_split(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    check_arity("split", &args, 1)?;
    if let (Value::Str(s), Value::Str(sep)) = (receiver, &args[0]) {
        let arr: Vec<Value> = s.split(sep)
            .map(|part| Value::Str(part.to_string()))
            .collect();
        // Don't forget to wrap the Vec in Rc<RefCell> for the Array type!
        Ok(Value::array(arr, false))
    } else {
        Err(InterpErr::Err("split() requires a String receiver and a String argument".to_string()))
    }
}

/// Joins all elements of an Array into a single String, separated by the given separator.
fn ext_join(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    check_arity("join", &args, 1)?;
    if let (Value::Array(arr, _), Value::Str(sep)) = (receiver, &args[0]) {
        let arr_ref = arr.borrow();
        let parts: Vec<String> = arr_ref.iter().map(|v| v.to_string()).collect();
        Ok(Value::Str(parts.join(sep)))
    } else {
        Err(InterpErr::Err("join() requires an Array receiver and a String separator argument".to_string()))
    }
}

/// Applies a given function to each element of the array, returning a new array.
fn ext_map(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    check_arity("map", &args, 1)?;
    if let (Value::Array(arr, _), func) = (receiver, args[0].clone()) {
        let arr_ref = arr.borrow().clone(); // Clone elements to avoid borrow conflicts during execution
        let mut new_arr = Vec::new();
        for item in arr_ref {
            let result = crate::interpreter::Environment::execute_function(func.clone(), vec![item])?;
            new_arr.push(result);
        }
        Ok(Value::array(new_arr, false))
    } else {
        Err(InterpErr::Err("map() requires an Array receiver and a function argument".to_string()))
    }
}

/// Filters the array, keeping only elements for which the function returns true.
fn ext_filter(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    check_arity("filter", &args, 1)?;
    if let (Value::Array(arr, _), func) = (receiver, args[0].clone()) {
        let arr_ref = arr.borrow().clone();
        let mut new_arr = Vec::new();
        for item in arr_ref {
            let result = crate::interpreter::Environment::execute_function(func.clone(), vec![item.clone()])?;
            // Use truthiness to evaluate the result
            if crate::interpreter::Environment::is_truthy_static(&result) {
                new_arr.push(item);
            }
        }
        Ok(Value::array(new_arr, false))
    } else {
        Err(InterpErr::Err("filter() requires an Array receiver and a function argument".to_string()))
    }
}

/// args() -> returns an Array of Strings containing the command-line arguments.
fn builtin_args(_args: Vec<Value>) -> InterpResult<Value> {
    let args: Vec<Value> = env::args().map(|s| Value::Str(s)).collect();
    Ok(Value::array(args, false))
}

/// exit(code) -> terminates the program immediately with the given exit code.
fn builtin_exit(args: Vec<Value>) -> InterpResult<Value> {
    let code = if let Some(Value::Number(c)) = args.get(0) { *c as i32 } else { 0 };
    std::process::exit(code);
}

/// shell(command) -> executes a command in the system shell and returns its output as a String.
fn builtin_shell(args: Vec<Value>) -> InterpResult<Value> {
    if let Some(Value::Str(cmd)) = args.get(0) {
        // Use 'sh' on Unix and 'cmd' on Windows for cross-platform compatibility
        let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
        let flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };
        
        let output = Command::new(shell)
            .arg(flag)
            .arg(cmd)
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                // Combine stdout and stderr to capture all output
                Ok(Value::Str(stdout + &stderr))
            }
            Err(e) => Err(InterpErr::Err(format!("Failed to execute shell command: {}", e))),
        }
    } else {
        Err(InterpErr::Err("shell() requires a String command argument".to_string()))
    }
}
