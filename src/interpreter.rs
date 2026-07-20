// src/interpreter.rs

use std::collections::HashMap;
use crate::ast::{BinOp, Expr, Stmt};

// Type for our built-in Rust functions
pub type BuiltinFn = fn(Vec<Value>) -> InterpResult<Value>;

// Values that can exist during program execution
#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Decimal(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
    Function(Vec<String>, Vec<Stmt>),
    Builtin(BuiltinFn),
    Null,
}

// Manual implementation of PartialEq that ignores function pointers (Builtin and Function)
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Decimal(a), Value::Decimal(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Null, Value::Null) => true,
            // Functions are simply treated as not equal because they cannot be safely compared
            _ => false, 
        }
    }
}

// Implementation of the Display/ToString logic for Value
impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Decimal(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                let formatted: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", formatted.join(", "))
            }
            Value::Function(_, _) => "<func>".to_string(),
            Value::Builtin(_) => "<builtin func>".to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

// Structure keeping value and the flag saying if it is mutable
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub value: Value,
    pub is_const: bool,
}

// Type for extension functions: takes the receiver object and a list of arguments
pub type ExtensionFn = fn(Value, Vec<Value>) -> InterpResult<Value>;

// Environment handles variable scopes (global vs local)
#[derive(Debug, Clone)]
pub struct Environment {
    pub vars: HashMap<String, VarInfo>,
    pub parent: Option<Box<Environment>>,
    pub extensions: HashMap<String, ExtensionFn>,
}

#[derive(Debug, Clone)]
pub enum InterpErr {
    Return(Value),
    Break,
    Continue,
    Err(String),
}

pub type InterpResult<T> = Result<T, InterpErr>;

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment { 
            vars: HashMap::new(), 
            parent: None,
            extensions: HashMap::new(),
        };
        
        // Load the anonymous standard library
        crate::stdlib::register_stdlib(&mut env);
        
        env
    }

    pub fn with_parent(parent: Environment) -> Self {
        // We pass the extensions down to the child scope
        Environment { 
            vars: HashMap::new(), 
            parent: Some(Box::new(parent.clone())),
            extensions: parent.extensions,
        }
    }

    pub fn get(&self, name: &str) -> Option<&VarInfo> {
        if let Some(v) = self.vars.get(name) {
            Some(v)
        } else if let Some(p) = &self.parent {
            p.get(name)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        if self.vars.contains_key(name) {
            return self.vars.get_mut(name);
        }
        if let Some(p) = &mut self.parent {
            return p.get_mut(name);
        }
        None
    }

    pub fn insert(&mut self, name: String, info: VarInfo) {
        self.vars.insert(name, info);
    }

    // Główna funkcja, która bierze listę instrukcji (AST) i je wykonuje
    pub fn run(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            match self.eval_stmt(stmt) {
                Ok(_) => {}
                Err(InterpErr::Err(e)) => return Err(e),
                Err(InterpErr::Return(_)) => return Err("Return outside of function".to_string()),
                Err(InterpErr::Break) => return Err("Break outside of loop".to_string()),
                Err(InterpErr::Continue) => return Err("Continue outside of loop".to_string()),
            }
        }
        Ok(())
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> InterpResult<()> {
        match stmt {
            Stmt::VarDecl(name, type_name, expr) => {
                let value = match expr {
                    Some(e) => self.eval_expr(e)?,
                    None => self.get_default_value(type_name)?,
                };
                self.insert(name.clone(), VarInfo { value, is_const: false });
            }

            Stmt::Let(name, type_name, expr) => {
                let value = match expr {
                    Some(e) => self.eval_expr(e)?,
                    None => self.get_default_value(type_name)?,
                };
                self.insert(name.clone(), VarInfo { value, is_const: true });
            }

            Stmt::Assign(name, expr) => {
                let value = self.eval_expr(expr)?;
                if let Some(info) = self.get_mut(name) {
                    if info.is_const {
                        return Err(InterpErr::Err(format!("Nie można zmienić wartości stałej '{}'", name)));
                    }
                    info.value = value;
                } else {
                    return Err(InterpErr::Err(format!("Zmienna '{}' nie jest zadeklarowana. Użyj 'var' lub 'let'.", name)));
                }
            }

            Stmt::ExprStmt(expr) => {
                // Evaluate expression and discard the result (used for side effects like print)
                self.eval_expr(expr)?;
            }

            Stmt::If(condition, if_body, else_body) => {
                let cond_value = self.eval_expr(condition)?;
                // Use truthiness instead of strict Bool matching
                if self.is_truthy(&cond_value) {
                    for s in if_body {
                        self.eval_stmt(s)?;
                    }
                } else {
                    for s in else_body {
                        self.eval_stmt(s)?;
                    }
                }
            }

            Stmt::Loop(var_name, start_expr, end_expr, body) => {
                let start_val = self.eval_expr(start_expr)?;
                let end_val = self.eval_expr(end_expr)?;
                if let (Value::Number(start), Value::Number(end)) = (start_val, end_val) {
                    'outer: for i in start..end {
                        self.insert(var_name.clone(), VarInfo { value: Value::Number(i), is_const: false });
                        for s in body {
                            match self.eval_stmt(s) {
                                Ok(_) => {}
                                Err(InterpErr::Continue) => continue 'outer,
                                Err(InterpErr::Break) => break 'outer,
                                Err(InterpErr::Return(v)) => return Err(InterpErr::Return(v)),
                                Err(InterpErr::Err(e)) => return Err(InterpErr::Err(e)),
                            }
                        }
                    }
                } else {
                    return Err(InterpErr::Err("Pętla 'loop' działa tylko z liczbami Numeric (całkowitymi)".to_string()));
                }
            }

            Stmt::IndexAssign(arr_expr, idx_expr, val_expr) => {
                let val = self.eval_expr(val_expr)?;
                let idx_val = self.eval_expr(idx_expr)?;
                if let Expr::Variable(name) = &**arr_expr {
                    if let Value::Number(idx) = idx_val {
                        if let Some(info) = self.get_mut(name) {
                            if let Value::Array(arr) = &mut info.value {
                                if idx < 0 || idx as usize >= arr.len() {
                                    return Err(InterpErr::Err(format!("Array index out of bounds: {}", idx)));
                                }
                                arr[idx as usize] = val;
                                return Ok(());
                            }
                            return Err(InterpErr::Err(format!("'{}' is not an array", name).to_string()));
                        }
                        return Err(InterpErr::Err(format!("Variable '{}' not defined", name).to_string()));
                    }
                    return Err(InterpErr::Err("Array index must be a number".to_string()));
                }
                return Err(InterpErr::Err("Invalid assignment target".to_string()))
            }

            Stmt::FuncDecl(name, params, body) => {
                let func_val = Value::Function(params.clone(), body.clone());
                self.insert(name.clone(), VarInfo { value: func_val, is_const: true });
            }

            Stmt::Return(expr) => {
                let val = match expr {
                    Some(e) => self.eval_expr(e)?,
                    None => Value::Null,
                };
                return Err(InterpErr::Return(val));
            }

            Stmt::Break => return Err(InterpErr::Break),

            Stmt::Continue => return Err(InterpErr::Continue),

            Stmt::Use(module_name) => {
                crate::modules::load_module(&mut self.extensions, module_name)?;
            }
        }
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> InterpResult<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Decimal(n) => Ok(Value::Decimal(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            
            Expr::Variable(name) => {
                self.get(name).map(|info| info.value.clone())
                    .ok_or_else(|| InterpErr::Err(format!("Zmienna '{}' nie jest zdefiniowana", name)))
            }

            Expr::Binary(left, op, right) => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;

                // String concatenation
                if let BinOp::Add = op {
                    if let (Value::Str(_), _) | (_, Value::Str(_)) = (&left_val, &right_val) {
                        let l_str = left_val.to_string();
                        let r_str = right_val.to_string();
                        return Ok(Value::Str(l_str + &r_str));
                    }
                }

                // Type promotion: if either side is Decimal, promote both to f64
                let (l_val, r_val) = match (left_val, right_val) {
                    (Value::Number(l), Value::Decimal(r)) => (Value::Decimal(l as f64), Value::Decimal(r)),
                    (Value::Decimal(l), Value::Number(r)) => (Value::Decimal(l), Value::Decimal(r as f64)),
                    other => other,
                };

                match (l_val, r_val) {
                    // Both are integers
                    (Value::Number(l), Value::Number(r)) => {
                        match op {
                            BinOp::Add => Ok(Value::Number(l + r)),
                            BinOp::Subtract => Ok(Value::Number(l - r)),
                            BinOp::Multiply => Ok(Value::Number(l * r)),
                            BinOp::Divide => {
                                if r == 0 {
                                    return Err(InterpErr::Err("Błąd wykonania: Dzielenie przez zero!".to_string()));
                                }
                                Ok(Value::Number(l / r)) // Integer division!
                            }
                            BinOp::Modulo => {
                                if r == 0 {
                                    return Err(InterpErr::Err("Błąd wykonania: Modulo przez zero!".to_string()));
                                }
                                Ok(Value::Number(l % r))
                            }
                            BinOp::Equals => Ok(Value::Bool(l == r)),
                            BinOp::NotEquals => Ok(Value::Bool(l != r)),
                            BinOp::GreaterThan => Ok(Value::Bool(l > r)),
                            BinOp::LessThan => Ok(Value::Bool(l < r)),
                            BinOp::GreaterEq => Ok(Value::Bool(l >= r)),
                            BinOp::LessEq => Ok(Value::Bool(l <= r)),
                        }
                    }
                    // Both are decimals (or promoted to decimals)
                    (Value::Decimal(l), Value::Decimal(r)) => {
                        match op {
                            BinOp::Add => Ok(Value::Decimal(l + r)),
                            BinOp::Subtract => Ok(Value::Decimal(l - r)),
                            BinOp::Multiply => Ok(Value::Decimal(l * r)),
                            BinOp::Divide => {
                                if r == 0.0 {
                                    return Err(InterpErr::Err("Błąd wykonania: Dzielenie przez zero!".to_string()));
                                }
                                Ok(Value::Decimal(l / r))
                            }
                            BinOp::Modulo => {
                                if r == 0.0 {
                                    return Err(InterpErr::Err("Błąd wykonania: Modulo przez zero!".to_string()));
                                }
                                Ok(Value::Decimal(l % r))
                            }
                            BinOp::Equals => Ok(Value::Bool(l == r)),
                            BinOp::NotEquals => Ok(Value::Bool(l != r)),
                            BinOp::GreaterThan => Ok(Value::Bool(l > r)),
                            BinOp::LessThan => Ok(Value::Bool(l < r)),
                            BinOp::GreaterEq => Ok(Value::Bool(l >= r)),
                            BinOp::LessEq => Ok(Value::Bool(l <= r)),
                        }
                    }
                    (Value::Str(l), Value::Str(r)) => {
                        match op {
                            BinOp::Add => Ok(Value::Str(l + &r)),
                            BinOp::Equals => Ok(Value::Bool(l == r)),
                            BinOp::NotEquals => Ok(Value::Bool(l != r)),
                            _ => Err(InterpErr::Err("Nieobsługiwany operator dla stringów".to_string())),
                        }
                    }
                    _ => Err(InterpErr::Err("Niekompatybilne typy w operacji binarnej".to_string())),
                }
            }

            Expr::Array(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.eval_expr(e)?);
                }
                Ok(Value::Array(vals))
            }

            Expr::IndexGet(arr_expr, idx_expr) => {
                let arr_val = self.eval_expr(arr_expr)?;
                let idx_val = self.eval_expr(idx_expr)?;
                if let (Value::Array(arr), Value::Number(idx)) = (arr_val, idx_val) {
                    if idx < 0 || idx as usize >= arr.len() {
                        return Err(InterpErr::Err(format!("Array index out of bounds: {}", idx)));
                    }
                    Ok(arr[idx as usize].clone())
                } else {
                    Err(InterpErr::Err("Can only index arrays with numbers".to_string()))
                }
            }

            Expr::Call(callee, args) => {
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_expr(arg)?);
                }

                if let Expr::Variable(name) = &**callee {
                    let func_val = self.get(name).map(|info| info.value.clone())
                        .ok_or_else(|| InterpErr::Err(format!("Function '{}' is not defined", name)))?;
                    
                    if let Value::Function(params, body) = func_val {
                        if params.len() != arg_vals.len() {
                            return Err(InterpErr::Err(format!("Expected {} arguments, got {}", params.len(), arg_vals.len())));
                        }
                        
                        let mut local_env = Environment::with_parent(self.clone());
                        
                        for (i, param_name) in params.iter().enumerate() {
                            local_env.insert(param_name.clone(), VarInfo { value: arg_vals[i].clone(), is_const: false });
                        }

                        for stmt in &body {
                            match local_env.eval_stmt(stmt) {
                                Ok(_) => {}
                                Err(InterpErr::Return(v)) => return Ok(v),
                                Err(InterpErr::Err(e)) => return Err(InterpErr::Err(e)),
                                // Break/Continue inside a function but outside a loop is an error
                                Err(InterpErr::Break) => return Err(InterpErr::Err("Break outside of loop".to_string())),
                                Err(InterpErr::Continue) => return Err(InterpErr::Err("Continue outside of loop".to_string())),
                            }
                        }
                        
                        return Ok(Value::Null);
                    } else if let Value::Builtin(func) = func_val {
                        // Simply call the Rust function pointer with the arguments
                        return func(arg_vals);
                    } else {
                        return Err(InterpErr::Err(format!("'{}' is not a function", name)));
                    }
                } else {
                    return Err(InterpErr::Err("Can only call functions by name".to_string()));
                }
            }

            // Method call
            Expr::MethodCall(obj_expr, method_name, args) => {
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_expr(arg)?);
                }

                let obj_val = self.eval_expr(obj_expr)?;

                // Wyjątek: metody mutujące (jak 'push' na tablicy) muszą być tu, 
                // bo operują na referencji do zmiennej w środowisku.
                if method_name == "push" {
                    if let Expr::Variable(name) = &**obj_expr {
                        if let Some(info) = self.get_mut(name) {
                            if let Value::Array(arr_mut) = &mut info.value {
                                arr_mut.push(arg_vals[0].clone());
                                return Ok(Value::Null);
                            }
                        }
                        return Err(InterpErr::Err("Can only push to an array variable".to_string()));
                    }
                    return Err(InterpErr::Err("Can only call push() on a variable".to_string()));
                }

                // Cała reszta to metody niewymagające mutacji (pure functions).
                // Szukamy ich w zarejestrowanych rozszerzeniach!
                if let Some(ext_fn) = self.extensions.get(method_name) {
                    return ext_fn(obj_val, arg_vals);
                }

                // Jeśli nie znaleziono metody
                match &obj_val {
                    Value::Array(_) => return Err(InterpErr::Err(format!("Method '{}' not supported on Array", method_name))),
                    Value::Str(_) => return Err(InterpErr::Err(format!("Method '{}' not supported on String", method_name))),
                    _ => return Err(InterpErr::Err(format!("Method '{}' not supported on this type", method_name))),
                }
            }

        }
    }

    // Helper to get default values for types
    fn get_default_value(&self, type_name: &Option<String>) -> InterpResult<Value> {
        match type_name {
            Some(t) => match t.as_str() {
                "Number" => Ok(Value::Number(0)),
                "Decimal" => Ok(Value::Decimal(0.0)),
                "String" => Ok(Value::Str("".to_string())),
                "Bool" => Ok(Value::Bool(false)),
                _ => Err(InterpErr::Err(format!("Unknown type: {}", t))),
            },
            None => Err(InterpErr::Err("Cannot infer default value without type".to_string())),
        }
    }

    // Helper to evaluate truthiness of any value
    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0,
            Value::Decimal(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Null => false,
            Value::Function(_, _) | Value::Builtin(_) => true,
        }
    }
}
