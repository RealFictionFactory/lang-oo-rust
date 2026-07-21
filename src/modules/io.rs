// src/modules/io.rs

//! Module for I/O related extension functions.
//! Implements a `File` object using a Dictionary under the hood,
//! keeping the interpreter core completely agnostic to file operations.

use crate::interpreter::{Environment, InterpErr, InterpResult, Value, VarInfo};
use std::collections::HashMap;
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

/// Registers I/O extension functions and the `file` constructor into the provided environment.
pub fn register(env: &Rc<RefCell<Environment>>) {
    // Global constructor for creating a File object (returns a Dict)
    Environment::insert(env, "file".to_string(), VarInfo { 
        value: Value::Builtin(builtin_file), 
        is_const: true,
        type_name: None // Type checking is bypassed, it returns a Dict
    });

    // Extension methods for File objects
    let mut env_mut = env.borrow_mut();
    env_mut.extensions.insert("read".to_string(), ext_file_read);
    env_mut.extensions.insert("write".to_string(), ext_file_write);
    env_mut.extensions.insert("append".to_string(), ext_file_append);
    env_mut.extensions.insert("exists".to_string(), ext_file_exists);
}

/// Helper to extract the file path from the Dict-based File object.
fn get_file_path(receiver: &Value) -> Result<String, InterpErr> {
    if let Value::Dict(map) = receiver {
        let map_ref = map.borrow();
        if let Some(Value::Str(path)) = map_ref.get("path") {
            return Ok(path.clone());
        }
    }
    Err(InterpErr::Err("Expected a File object".to_string()))
}

/// Creates a new File object (Dict) from a path string.
fn builtin_file(args: Vec<Value>) -> InterpResult<Value> {
    if let Some(Value::Str(path)) = args.get(0) {
        let mut map = HashMap::new();
        map.insert("__type__".to_string(), Value::Str("File".to_string()));
        map.insert("path".to_string(), Value::Str(path.clone()));
        Ok(Value::Dict(Rc::new(RefCell::new(map))))
    } else {
        Err(InterpErr::Err("file() requires a String path argument".to_string()))
    }
}

/// Reads the entire contents of the file into a String.
fn ext_file_read(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    let path = get_file_path(&receiver)?;
    match fs::read_to_string(&path) {
        Ok(content) => Ok(Value::Str(content)),
        Err(e) => Err(InterpErr::Err(format!("Failed to read file '{}': {}", path, e))),
    }
}

/// Overwrites the file with the given String content.
fn ext_file_write(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    let path = get_file_path(&receiver)?;
    if let Some(Value::Str(content)) = args.get(0) {
        match fs::write(&path, content) {
            Ok(_) => Ok(Value::Null),
            Err(e) => Err(InterpErr::Err(format!("Failed to write to file '{}': {}", path, e))),
        }
    } else {
        Err(InterpErr::Err("write() requires a String argument".to_string()))
    }
}

/// Appends the given String content to the end of the file.
fn ext_file_append(receiver: Value, args: Vec<Value>) -> InterpResult<Value> {
    let path = get_file_path(&receiver)?;
    if let Some(Value::Str(content)) = args.get(0) {
        let mut file = match fs::OpenOptions::new().append(true).create(true).open(&path) {
            Ok(f) => f,
            Err(e) => return Err(InterpErr::Err(format!("Failed to open file '{}' for appending: {}", path, e))),
        };
        match file.write_all(content.as_bytes()) {
            Ok(_) => Ok(Value::Null),
            Err(e) => Err(InterpErr::Err(format!("Failed to append to file '{}': {}", path, e))),
        }
    } else {
        Err(InterpErr::Err("append() requires a String argument".to_string()))
    }
}

/// Checks if the file exists on the filesystem.
fn ext_file_exists(receiver: Value, _args: Vec<Value>) -> InterpResult<Value> {
    let path = get_file_path(&receiver)?;
    Ok(Value::Bool(Path::new(&path).exists()))
}
