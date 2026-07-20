// src/modules/io.rs

//! Module for I/O related extension functions.
//! Currently empty as `input` was moved to the global standard library.
//! Future I/O extensions (e.g., file reading/writing) can be registered here.

use crate::interpreter::ExtensionFn;
use std::collections::HashMap;

/// Registers I/O extension functions into the provided extensions map.
/// Currently a placeholder for future functionality.
pub fn register(_exts: &mut HashMap<String, ExtensionFn>) {
    // Intentionally empty.
    // Previously hosted the `input` extension, which is now a global builtin function.
}
