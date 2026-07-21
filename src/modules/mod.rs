// src/modules/mod.rs

/// Module system for loading external or standard library extensions.
pub mod io;
use crate::interpreter::{Environment, InterpErr};
use std::rc::Rc;
use std::cell::RefCell;

/// Loads a specified module by name and registers its extension functions 
/// into the provided environment.
/// Returns an error if the requested module does not exist.
pub fn load_module(env: &Rc<RefCell<Environment>>, name: &str) -> Result<(), InterpErr> {
    match name {
        "io" => io::register(env),
        _ => return Err(InterpErr::Err(format!("Unknown module: {}", name))),
    }
    Ok(())
}
