// src/modules/mod.rs

pub mod io;
use crate::interpreter::{ExtensionFn, InterpErr};
use std::collections::HashMap;

pub fn load_module(exts: &mut HashMap<String, ExtensionFn>, name: &str) -> Result<(), InterpErr> {
    match name {
        "io" => io::register(exts),
        _ => return Err(InterpErr::Err(format!("Unknown module: {}", name))),
    }
    Ok(())
}
