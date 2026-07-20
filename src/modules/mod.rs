// src/modules/mod.rs

/// Module system for loading external or standard library extensions.
pub mod io;
use crate::interpreter::{ExtensionFn, InterpErr};
use std::collections::HashMap;

/// Loads a specified module by name and registers its extension functions 
/// into the provided extensions HashMap.
/// Returns an error if the requested module does not exist.
pub fn load_module(exts: &mut HashMap<String, ExtensionFn>, name: &str) -> Result<(), InterpErr> {
    match name {
        "io" => io::register(exts),
        _ => return Err(InterpErr::Err(format!("Unknown module: {}", name))),
    }
    Ok(())
}
