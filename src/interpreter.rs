// src/interpreter.rs

use std::collections::HashMap;
use crate::ast::{BinOp, Expr, Stmt, UnOp};

/// Type alias for built-in Rust functions used in the standard library.
pub type BuiltinFn = fn(Vec<Value>) -> InterpResult<Value>;

/// Represents all possible runtime values in the language.
#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Decimal(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
    Dict(HashMap<String, Value>),
    Function(Vec<String>, Vec<Stmt>),
    Builtin(BuiltinFn),
    Null,
}

// Manual implementation of PartialEq that ignores function pointers (Builtin and Function).
// Functions are simply treated as not equal because they cannot be safely compared by value.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Decimal(a), Value::Decimal(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Dict(a), Value::Dict(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false, 
        }
    }
}

/// Implementation of the Display/ToString logic for Value.
impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Decimal(n) => {
                // Use Rust's default formatting, but ensure we always show a decimal point
                // e.g., 1.0 -> "1.0", 3.14 -> "3.14"
                let s = format!("{}", n);

                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                let formatted: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", formatted.join(", "))
            }
            Value::Dict(map) => {
                let formatted: Vec<String> = map.iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", formatted.join(", "))
            }
            Value::Function(_, _) => "<func>".to_string(),
            Value::Builtin(_) => "<builtin func>".to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

/// Structure keeping a value and a flag indicating whether it is mutable.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub value: Value,
    pub is_const: bool,
}

/// Type alias for extension functions: takes the receiver object and a list of arguments.
pub type ExtensionFn = fn(Value, Vec<Value>) -> InterpResult<Value>;

/// Environment handles variable scopes (global vs local) and stores registered extensions.
#[derive(Debug, Clone)]
pub struct Environment {
    pub vars: HashMap<String, VarInfo>,
    pub parent: Option<Box<Environment>>,
    pub extensions: HashMap<String, ExtensionFn>,
}

/// Represents control flow interruptions or runtime errors during execution.
#[derive(Debug, Clone)]
pub enum InterpErr {
    Return(Value),
    Break,
    Continue,
    Err(String),
}

pub type InterpResult<T> = Result<T, InterpErr>;

impl Environment {
    /// Creates a new global environment and loads the standard library into it.
    pub fn new() -> Self {
        let mut env = Environment { 
            vars: HashMap::new(), 
            parent: None,
            extensions: HashMap::new(),
        };
        
        // Load the standard library (global functions and extensions)
        crate::stdlib::register_stdlib(&mut env);
        
        env
    }

    /// Creates a child environment with a parent scope. 
    /// Extensions are passed down to the child scope.
    pub fn with_parent(parent: Environment) -> Self {
        Environment { 
            vars: HashMap::new(), 
            parent: Some(Box::new(parent.clone())),
            extensions: parent.extensions,
        }
    }

    /// Looks up a variable in the current scope, recursively checking parent scopes.
    pub fn get(&self, name: &str) -> Option<&VarInfo> {
        if let Some(v) = self.vars.get(name) {
            Some(v)
        } else if let Some(p) = &self.parent {
            p.get(name)
        } else {
            None
        }
    }

    /// Looks up a variable for mutation in the current scope, recursively checking parent scopes.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        if self.vars.contains_key(name) {
            return self.vars.get_mut(name);
        }
        if let Some(p) = &mut self.parent {
            return p.get_mut(name);
        }
        None
    }

    /// Inserts a variable into the current environment scope.
    pub fn insert(&mut self, name: String, info: VarInfo) {
        self.vars.insert(name, info);
    }

    /// Main execution function. Takes a list of AST statements and executes them.
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

    /// Evaluates a single statement.
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
                        return Err(InterpErr::Err(format!("Cannot change value of constant '{}'", name)));
                    }
                    info.value = value;
                } else {
                    return Err(InterpErr::Err(format!("Variable '{}' is not declared. Use 'var' or 'let'.", name)));
                }
            }

            Stmt::ExprStmt(expr) => {
                // Evaluate expression and discard the result (used for side effects like print)
                self.eval_expr(expr)?;
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
                    return Err(InterpErr::Err("'loop' only works with Numeric (integer) values".to_string()));
                }
            }

            Stmt::IndexAssign(container_expr, idx_expr, val_expr) => {
                let val = self.eval_expr(val_expr)?;
                let idx_val = self.eval_expr(idx_expr)?;
                if let Expr::Variable(name) = &**container_expr {
                    if let Some(info) = self.get_mut(name) {
                        match (&mut info.value, idx_val) {
                            (Value::Array(arr), Value::Number(idx)) => {
                                if idx < 0 || idx as usize >= arr.len() {
                                    return Err(InterpErr::Err(format!("Array index out of bounds: {}", idx)));
                                }
                                arr[idx as usize] = val;
                                return Ok(());
                            }
                            (Value::Dict(map), Value::Str(key)) => {
                                map.insert(key, val);
                                return Ok(());
                            }
                            _ => return Err(InterpErr::Err(format!("'{}' is not an array or dict", name).to_string()))
                        }
                    }
                    return Err(InterpErr::Err(format!("Variable '{}' not defined", name).to_string()));
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

            // 'until' działa jak break, ale tylko jeśli warunek jest prawdziwy
            Stmt::Until(condition) => {
                let cond_val = self.eval_expr(condition)?;
                if self.is_truthy(&cond_val) {
                    return Err(InterpErr::Break);
                }
            }

            // Nieskończona pętla: loop { ... }
            Stmt::LoopBlock(body) => {
                loop {
                    let mut should_break = false;
                    for s in body {
                        match self.eval_stmt(s) {
                            Ok(_) => {}
                            // Until i Break zwracają ten sam błąd
                            Err(InterpErr::Break) => {
                                should_break = true;
                                break;
                            }
                            Err(InterpErr::Continue) => {
                                // Continue przerywa obecną iterację pętli for
                                break; 
                            }
                            Err(InterpErr::Return(v)) => return Err(InterpErr::Return(v)),
                            Err(InterpErr::Err(e)) => return Err(InterpErr::Err(e)),
                        }
                    }
                    if should_break {
                        break;
                    }
                }
            }

            // Pętla po tablicy: loop element in array { ... }
            Stmt::LoopIn(var_name, iterable_expr, body) => {
                let iterable_val = self.eval_expr(iterable_expr)?;
                if let Value::Array(arr) = iterable_val {
                    'outer: for element in arr {
                        self.insert(var_name.clone(), VarInfo { value: element, is_const: false });
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
                    return Err(InterpErr::Err("'loop in' only works with Arrays".to_string()));
                }
            }

            Stmt::Break => return Err(InterpErr::Break),

            Stmt::Continue => return Err(InterpErr::Continue),

            Stmt::Use(module_name) => {
                crate::modules::load_module(&mut self.extensions, module_name)?;
            }
        }
        Ok(())
    }

    /// Evaluates an expression and returns its computed Value.
    fn eval_expr(&mut self, expr: &Expr) -> InterpResult<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),

            Expr::Decimal(n) => Ok(Value::Decimal(*n)),

            Expr::Str(s) => Ok(Value::Str(s.clone())),

            Expr::Bool(b) => Ok(Value::Bool(*b)),
            
            Expr::Variable(name) => {
                self.get(name).map(|info| info.value.clone())
                    .ok_or_else(|| InterpErr::Err(format!("Variable '{}' is not defined", name)))
            }

            Expr::Binary(left, op, right) => {
                // Short-circuit evaluation for logical operators
                if let BinOp::And = op {
                    let left_val = self.eval_expr(left)?;
                    if !self.is_truthy(&left_val) {
                        return Ok(Value::Bool(false));
                    }
                    let right_val = self.eval_expr(right)?;
                    return Ok(Value::Bool(self.is_truthy(&right_val)));
                }
                
                if let BinOp::Or = op {
                    let left_val = self.eval_expr(left)?;
                    if self.is_truthy(&left_val) {
                        return Ok(Value::Bool(true));
                    }
                    let right_val = self.eval_expr(right)?;
                    return Ok(Value::Bool(self.is_truthy(&right_val)));
                }

                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;

                // String concatenation: if either side is a String, concatenate them
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
                                    return Err(InterpErr::Err("Runtime error: Division by zero!".to_string()));
                                }
                                Ok(Value::Number(l / r)) // Integer division!
                            }
                            BinOp::Modulo => {
                                if r == 0 {
                                    return Err(InterpErr::Err("Runtime error: Modulo by zero!".to_string()));
                                }
                                Ok(Value::Number(l % r))
                            }
                            BinOp::Equals => Ok(Value::Bool(l == r)),
                            BinOp::NotEquals => Ok(Value::Bool(l != r)),
                            BinOp::GreaterThan => Ok(Value::Bool(l > r)),
                            BinOp::LessThan => Ok(Value::Bool(l < r)),
                            BinOp::GreaterEq => Ok(Value::Bool(l >= r)),
                            BinOp::LessEq => Ok(Value::Bool(l <= r)),
                            BinOp::And | BinOp::Or => Err(InterpErr::Err("Logical operators handled earlier".to_string())),
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
                                    return Err(InterpErr::Err("Runtime error: Division by zero!".to_string()));
                                }
                                Ok(Value::Decimal(l / r))
                            }
                            BinOp::Modulo => {
                                if r == 0.0 {
                                    return Err(InterpErr::Err("Runtime error: Modulo by zero!".to_string()));
                                }
                                Ok(Value::Decimal(l % r))
                            }
                            BinOp::Equals => Ok(Value::Bool(l == r)),
                            BinOp::NotEquals => Ok(Value::Bool(l != r)),
                            BinOp::GreaterThan => Ok(Value::Bool(l > r)),
                            BinOp::LessThan => Ok(Value::Bool(l < r)),
                            BinOp::GreaterEq => Ok(Value::Bool(l >= r)),
                            BinOp::LessEq => Ok(Value::Bool(l <= r)),
                            BinOp::And | BinOp::Or => Err(InterpErr::Err("Logical operators handled earlier".to_string())),
                        }
                    }
                    (Value::Str(l), Value::Str(r)) => {
                        match op {
                            BinOp::Add => Ok(Value::Str(l + &r)),
                            BinOp::Equals => Ok(Value::Bool(l == r)),
                            BinOp::NotEquals => Ok(Value::Bool(l != r)),
                            _ => Err(InterpErr::Err("Unsupported operator for strings".to_string())),
                        }
                    }
                    _ => Err(InterpErr::Err("Incompatible types in binary operation".to_string())),
                }
            }

            Expr::Unary(op, right) => {
                let right_val = self.eval_expr(right)?;
                match op {
                    UnOp::Negate => {
                        match right_val {
                            Value::Number(n) => Ok(Value::Number(-n)),
                            Value::Decimal(n) => Ok(Value::Decimal(-n)),
                            _ => Err(InterpErr::Err("Unary '-' can only be applied to Number or Decimal".to_string())),
                        }
                    }
                    UnOp::Not => {
                        // Use truthiness to evaluate 'not'
                        Ok(Value::Bool(!self.is_truthy(&right_val)))
                    }
                }
            }

            Expr::Array(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.eval_expr(e)?);
                }
                Ok(Value::Array(vals))
            }

            Expr::Dict(pairs) => {
                let mut map = HashMap::new();
                for (k_expr, v_expr) in pairs {
                    let k_val = self.eval_expr(k_expr)?;
                    let v_val = self.eval_expr(v_expr)?;
                    
                    if let Value::Str(key) = k_val {
                        map.insert(key, v_val);
                    } else {
                        return Err(InterpErr::Err("Dictionary keys must evaluate to String".to_string()));
                    }
                }
                Ok(Value::Dict(map))
            }

            Expr::IndexGet(container_expr, idx_expr) => {
                let container_val = self.eval_expr(container_expr)?;
                let idx_val = self.eval_expr(idx_expr)?;
                
                match (container_val, idx_val) {
                    (Value::Array(arr), Value::Number(idx)) => {
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(InterpErr::Err(format!("Array index out of bounds: {}", idx)));
                        }
                        Ok(arr[idx as usize].clone())
                    }
                    (Value::Dict(map), Value::Str(key)) => { // NOWOŚĆ
                        map.get(&key).cloned()
                            .ok_or_else(|| InterpErr::Err(format!("Key '{}' not found in dictionary", key)))
                    }
                    _ => Err(InterpErr::Err("Can only index arrays with numbers or dicts with strings".to_string()))
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

            Expr::MethodCall(obj_expr, method_name, args) => {
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_expr(arg)?);
                }

                let obj_val = self.eval_expr(obj_expr)?;

                // Exception: mutating methods (like 'push' on an array) must be handled here, 
                // because they operate on a mutable reference to the variable in the environment.
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

                // The rest are non-mutating methods (pure functions).
                // We look them up in the registered extensions!
                if let Some(ext_fn) = self.extensions.get(method_name) {
                    return ext_fn(obj_val, arg_vals);
                }

                // If the method was not found
                match &obj_val {
                    Value::Array(_) => return Err(InterpErr::Err(format!("Method '{}' not supported on Array", method_name))),
                    Value::Str(_) => return Err(InterpErr::Err(format!("Method '{}' not supported on String", method_name))),
                    _ => return Err(InterpErr::Err(format!("Method '{}' not supported on this type", method_name))),
                }
            }

            // Executes the run_body. If a runtime error (Err) occurs, it catches it,
            // binds the error message to the err_var (if provided) in the current scope,
            // and executes the catch_body. Other control flow errors (Return, Break, Continue) propagate up.
            Expr::ExecuteCatch(run_body, err_var, catch_body) => {
                match self.eval_block_as_expr(run_body) {
                    Ok(val) => return Ok(val),
                    Err(InterpErr::Err(msg)) => {
                        if let Some(var_name) = err_var {
                            self.insert(var_name.clone(), VarInfo { 
                                value: Value::Str(msg), 
                                is_const: true 
                            });
                        }
                        return self.eval_block_as_expr(catch_body);
                    }
                    Err(other_err) => {
                        return Err(other_err);
                    }
                }
            }

            // Evaluates the condition. If truthy, evaluates and returns the if_body.
            // Otherwise, evaluates and returns the else_body.
            // Works in the current scope to allow variable reassignment inside the blocks.
            Expr::If(condition, if_body, else_body) => {
                let cond_value = self.eval_expr(condition)?;
                if self.is_truthy(&cond_value) {
                    return self.eval_block_as_expr(if_body);
                } else {
                    return self.eval_block_as_expr(else_body);
                }
            }
        }
    }

    /// Helper to get default values for types (e.g., when declaring a variable without an initial value).
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

    /// Helper to evaluate the truthiness of any value.
    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0,
            Value::Decimal(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Null => false,
            Value::Function(_, _) | Value::Builtin(_) => true,
            Value::Dict(hash_map) => !hash_map.is_empty(),
        }
    }

    /// Helper to evaluate a block of statements as an expression.
    /// Returns the value of the last expression statement in the block.
    /// If a Return statement is encountered, it returns Err(InterpErr::Return(v)), 
    /// which correctly propagates up via the `?` operator so the function exits properly.
    fn eval_block_as_expr(&mut self, stmts: &[Stmt]) -> InterpResult<Value> {
        let mut last_val = Value::Null;
        for stmt in stmts {
            match stmt {
                // If it's an expression, save its value
                Stmt::ExprStmt(expr) => last_val = self.eval_expr(expr)?,
                // Execute other statements (including Return, VarDecl, etc.) normally.
                // If it's a Return, it will return Err(InterpErr::Return(v)), which will
                // propagate up via the `?` operator!
                _ => self.eval_stmt(stmt)?,
            }
        }
        Ok(last_val)
    }
}
