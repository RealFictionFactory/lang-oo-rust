// src/interpreter.rs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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
    Array(Rc<RefCell<Vec<Value>>>),             // Shared mutable array
    Dict(Rc<RefCell<HashMap<String, Value>>>),  // Shared mutable dict
    // Stores Rc<RefCell<Environment>> to allow closures to share state with their definition scope.
    // Parameters and body are shared via Rc: Value is cloned on every variable lookup, and
    // deep-copying the body there made each call cost O(size of the function's source).
    Function(Rc<Vec<String>>, Rc<Vec<Stmt>>, Rc<RefCell<Environment>>),
    Builtin(BuiltinFn),
    Null,
}

// Manual implementation of PartialEq that ignores function pointers and environments.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Decimal(a), Value::Decimal(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                let a_ref = a.borrow();
                let b_ref = b.borrow();
                a_ref.len() == b_ref.len() && a_ref.iter().zip(b_ref.iter()).all(|(x, y)| x == y)
            }
            (Value::Dict(a), Value::Dict(b)) => {
                let a_ref = a.borrow();
                let b_ref = b.borrow();
                a_ref.len() == b_ref.len() && a_ref.iter().all(|(k, v)| b_ref.get(k).map_or(false, |bv| v == bv))
            }
            (Value::Null, Value::Null) => true,
            (Value::Function(a, b, _), Value::Function(c, d, _)) => a == c && b == d,
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
                let s = format!("{}", n);
                if s.contains('.') { s } else { format!("{}.0", s) }
            }
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                let arr_ref = arr.borrow();
                let formatted: Vec<String> = arr_ref.iter().map(|v| v.to_string()).collect();
                format!("[{}]", formatted.join(", "))
            }
            Value::Dict(map) => {
                let map_ref = map.borrow();
                let formatted: Vec<String> = map_ref.iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", formatted.join(", "))
            }
            Value::Function(_, _, _) => "<func>".to_string(),
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
    pub type_name: Option<String>,
}

/// Type alias for extension functions: takes the receiver object and a list of arguments.
pub type ExtensionFn = fn(Value, Vec<Value>) -> InterpResult<Value>;

/// Environment handles variable scopes (global vs local) and stores registered extensions.
/// Wrapped in Rc<RefCell<T>> to allow shared ownership and interior mutability for closures.
#[derive(Debug)]
pub struct Environment {
    pub vars: HashMap<String, VarInfo>,
    pub parent: Option<Rc<RefCell<Environment>>>,
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
    pub fn new() -> Rc<RefCell<Environment>> {
        let env = Environment { 
            vars: HashMap::new(), 
            parent: None,
            extensions: HashMap::new(),
        };
        let rc_env = Rc::new(RefCell::new(env));
        
        // Load the standard library (global functions and extensions)
        crate::stdlib::register_stdlib(&rc_env);
        
        rc_env
    }

    /// Creates a child environment with a parent scope.
    /// The extension registry starts empty and is resolved through the parent chain by
    /// `find_extension`, so creating a scope stays cheap no matter how many are registered.
    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            vars: HashMap::new(),
            parent: Some(parent),
            extensions: HashMap::new(),
        }))
    }

    /// Looks up an extension method by name, walking up the scope chain.
    /// Returns the function pointer by value so no borrow outlives the lookup.
    fn find_extension(env: &Rc<RefCell<Environment>>, name: &str) -> Option<ExtensionFn> {
        let env_ref = env.borrow();
        if let Some(f) = env_ref.extensions.get(name) {
            return Some(*f);
        }
        match &env_ref.parent {
            Some(p) => Self::find_extension(p, name),
            None => None,
        }
    }

    /// Looks up a variable in the current scope, recursively checking parent scopes.
    pub fn get(env: &Rc<RefCell<Environment>>, name: &str) -> Option<VarInfo> {
        let env_ref = env.borrow();
        if let Some(v) = env_ref.vars.get(name) {
            return Some(v.clone());
        }
        if let Some(p) = &env_ref.parent {
            return Self::get(p, name);
        }
        None
    }

    /// Finds the environment scope where a variable is defined.
    fn find_env_with_var(env: &Rc<RefCell<Environment>>, name: &str) -> Option<Rc<RefCell<Environment>>> {
        if env.borrow().vars.contains_key(name) {
            return Some(Rc::clone(env));
        }
        if let Some(p) = &env.borrow().parent {
            return Self::find_env_with_var(p, name);
        }
        None
    }

    /// Assigns a new value to an existing variable, respecting const and type checks.
    fn assign_var(env: &Rc<RefCell<Environment>>, name: &str, value: Value) -> InterpResult<()> {
        if let Some(var_env) = Self::find_env_with_var(env, name) {
            let mut env_mut = var_env.borrow_mut();
            let info = env_mut.vars.get_mut(name).unwrap();
            if info.is_const {
                return Err(InterpErr::Err(format!("Cannot change value of constant '{}'", name)));
            }
            if let Some(t) = &info.type_name {
                if !Self::value_matches_type(t, &value) {
                    return Err(InterpErr::Err(format!("Type mismatch: cannot assign {} to variable of type {}", Self::value_type_name(&value), t)));
                }
            }
            info.value = value;
            return Ok(());
        }
        Err(InterpErr::Err(format!("Variable '{}' is not declared. Use 'var' or 'let'.", name)))
    }

    /// Inserts a variable into the current environment scope.
    pub fn insert(env: &Rc<RefCell<Environment>>, name: String, info: VarInfo) {
        env.borrow_mut().vars.insert(name, info);
    }

    /// Main execution function. Takes a list of AST statements and executes them.
    pub fn run(env: &Rc<RefCell<Environment>>, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            match Self::eval_stmt(env, stmt) {
                Ok(_) => {}
                Err(InterpErr::Err(e)) => return Err(e),
                Err(InterpErr::Return(_)) => return Err("Return outside of function".to_string()),
                Err(InterpErr::Break) => return Err("Break outside of loop".to_string()),
                Err(InterpErr::Continue) => return Err("Continue outside of loop".to_string()),
            }
        }
        Ok(())
    }

    /// Helper to execute a function value (used by stdlib for .map, .filter, etc.)
    /// Takes a Value (Function or Builtin) and a list of arguments.
    pub fn execute_function(func: Value, args: Vec<Value>) -> InterpResult<Value> {
        if let Value::Function(params, body, closure_env) = func {
            if params.len() != args.len() {
                return Err(InterpErr::Err(format!("Expected {} arguments, got {}", params.len(), args.len())));
            }
            
            let local_env = Environment::with_parent(closure_env);
            for (i, param_name) in params.iter().enumerate() {
                Self::insert(&local_env, param_name.clone(), VarInfo { value: args[i].clone(), is_const: false, type_name: None });
            }

            for stmt in body.iter() {
                match Self::eval_stmt(&local_env, stmt) {
                    Ok(_) => {}
                    Err(InterpErr::Return(v)) => return Ok(v),
                    Err(InterpErr::Err(e)) => return Err(InterpErr::Err(e)),
                    Err(InterpErr::Break) => return Err(InterpErr::Err("Break outside of loop".to_string())),
                    Err(InterpErr::Continue) => return Err(InterpErr::Err("Continue outside of loop".to_string())),
                }
            }
            return Ok(Value::Null);
        } else if let Value::Builtin(b_func) = func {
            return b_func(args);
        } else {
            return Err(InterpErr::Err("Value is not callable".to_string()));
        }
    }

    /// Evaluates a single statement.
    fn eval_stmt(env: &Rc<RefCell<Environment>>, stmt: &Stmt) -> InterpResult<()> {
        match stmt {
            Stmt::VarDecl(name, type_name, expr) => {
                let value = match expr {
                    Some(e) => Self::eval_expr(env, e)?,
                    None => Self::get_default_value(type_name)?,
                };
                if let Some(t) = type_name {
                    if !Self::value_matches_type(t, &value) {
                        return Err(InterpErr::Err(format!("Type mismatch: expected '{}', got {}", t, Self::value_type_name(&value))));
                    }
                }
                Self::insert(env, name.clone(), VarInfo { value, is_const: false, type_name: type_name.clone() });
            }

            Stmt::Let(name, type_name, expr) => {
                let value = match expr {
                    Some(e) => Self::eval_expr(env, e)?,
                    None => Self::get_default_value(type_name)?,
                };
                if let Some(t) = type_name {
                    if !Self::value_matches_type(t, &value) {
                        return Err(InterpErr::Err(format!("Type mismatch: expected '{}', got {}", t, Self::value_type_name(&value))));
                    }
                }
                Self::insert(env, name.clone(), VarInfo { value, is_const: true, type_name: type_name.clone() });
            }

            Stmt::Assign(name, expr) => {
                let value = Self::eval_expr(env, expr)?;
                Self::assign_var(env, name, value)?;
            }

            Stmt::ExprStmt(expr) => {
                Self::eval_expr(env, expr)?;
            }

            Stmt::Loop(var_name, start_expr, end_expr, body) => {
                let start_val = Self::eval_expr(env, start_expr)?;
                let end_val = Self::eval_expr(env, end_expr)?;
                if let (Value::Number(start), Value::Number(end)) = (start_val, end_val) {
                    // The iterator and anything the body declares live in a loop-owned scope,
                    // so they neither clobber nor outlive the surrounding bindings.
                    let loop_env = Self::with_parent(Rc::clone(env));
                    'outer: for i in start..end {
                        Self::insert(&loop_env, var_name.clone(), VarInfo { value: Value::Number(i), is_const: false, type_name: None });
                        for s in body {
                            match Self::eval_stmt(&loop_env, s) {
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
                let val = Self::eval_expr(env, val_expr)?;
                let idx_val = Self::eval_expr(env, idx_expr)?;
                if let Expr::Variable(name) = &**container_expr {
                    if let Some(var_env) = Self::find_env_with_var(env, name) {
                        let mut env_mut = var_env.borrow_mut();
                        if let Some(info) = env_mut.vars.get_mut(name) {
                            // A `let` binding protects its container from indexed writes.
                            if info.is_const {
                                return Err(InterpErr::Err(format!("Cannot modify '{}': it is declared with 'let'", name)));
                            }
                            match (&mut info.value, idx_val) {
                                (Value::Array(arr), Value::Number(idx)) => {
                                    let mut arr_mut = arr.borrow_mut();
                                    if idx < 0 || idx as usize >= arr_mut.len() {
                                        return Err(InterpErr::Err(format!("Array index out of bounds: {}", idx)));
                                    }
                                    arr_mut[idx as usize] = val;
                                    return Ok(());
                                }
                                (Value::Dict(map), Value::Str(key)) => {
                                    let mut map_mut = map.borrow_mut();
                                    map_mut.insert(key, val);
                                    return Ok(());
                                }
                                _ => return Err(InterpErr::Err(format!("'{}' is not an array or dict", name).to_string()))
                            }
                        }
                    }
                    return Err(InterpErr::Err(format!("Variable '{}' not defined", name).to_string()));
                }
                return Err(InterpErr::Err("Invalid assignment target".to_string()))
            }

            Stmt::FuncDecl(name, params, body) => {
                // Capture the current environment by cloning the Rc (cheap, shared ownership).
                // Params and body are Rc too, so this clones three pointers, not the AST.
                let func_val = Value::Function(Rc::clone(params), Rc::clone(body), Rc::clone(env));
                Self::insert(env, name.clone(), VarInfo { value: func_val, is_const: true, type_name: None });
            }

            Stmt::Return(expr) => {
                let val = match expr {
                    Some(e) => Self::eval_expr(env, e)?,
                    None => Value::Null,
                };
                return Err(InterpErr::Return(val));
            }

            Stmt::Until(condition) => {
                let cond_val = Self::eval_expr(env, condition)?;
                if Self::is_truthy_static(&cond_val) {
                    return Err(InterpErr::Break);
                }
            }

            Stmt::LoopBlock(body) => {
                // Loop-owned scope, as in the other two loop forms.
                let loop_env = Self::with_parent(Rc::clone(env));
                loop {
                    let mut should_break = false;
                    for s in body {
                        match Self::eval_stmt(&loop_env, s) {
                            Ok(_) => {}
                            Err(InterpErr::Break) => {
                                should_break = true;
                                break;
                            }
                            Err(InterpErr::Continue) => {
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

            Stmt::LoopIn(var_name, iterable_expr, body) => {
                let iterable_val = Self::eval_expr(env, iterable_expr)?;
                if let Value::Array(arr) = iterable_val {
                    let arr_clone = arr.borrow().clone(); // Clone elements to avoid borrow issues during loop
                    // Loop-owned scope, as in the range loop above.
                    let loop_env = Self::with_parent(Rc::clone(env));
                    'outer: for element in arr_clone {
                        Self::insert(&loop_env, var_name.clone(), VarInfo { value: element, is_const: false, type_name: None });
                        for s in body {
                            match Self::eval_stmt(&loop_env, s) {
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
                crate::modules::load_module(env, module_name)?;
            }
        }
        Ok(())
    }

    /// Evaluates an expression and returns its computed Value.
    fn eval_expr(env: &Rc<RefCell<Environment>>, expr: &Expr) -> InterpResult<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),

            Expr::Decimal(n) => Ok(Value::Decimal(*n)),

            Expr::Str(s) => Ok(Value::Str(s.clone())),

            Expr::Bool(b) => Ok(Value::Bool(*b)),
            
            Expr::Variable(name) => {
                Self::get(env, name).map(|info| info.value)
                    .ok_or_else(|| InterpErr::Err(format!("Variable '{}' is not defined", name)))
            }

            Expr::Lambda(params, body) => {
                // Closures capture the Rc to the current environment
                Ok(Value::Function(Rc::clone(params), Rc::clone(body), Rc::clone(env)))
            }

            Expr::Binary(left, op, right) => {
                if let BinOp::And = op {
                    let left_val = Self::eval_expr(env, left)?;
                    if !Self::is_truthy_static(&left_val) { return Ok(Value::Bool(false)); }
                    let right_val = Self::eval_expr(env, right)?;
                    return Ok(Value::Bool(Self::is_truthy_static(&right_val)));
                }
                
                if let BinOp::Or = op {
                    let left_val = Self::eval_expr(env, left)?;
                    if Self::is_truthy_static(&left_val) { return Ok(Value::Bool(true)); }
                    let right_val = Self::eval_expr(env, right)?;
                    return Ok(Value::Bool(Self::is_truthy_static(&right_val)));
                }

                let left_val = Self::eval_expr(env, left)?;
                let right_val = Self::eval_expr(env, right)?;

                if let BinOp::Add = op {
                    if let (Value::Str(_), _) | (_, Value::Str(_)) = (&left_val, &right_val) {
                        let l_str = left_val.to_string();
                        let r_str = right_val.to_string();
                        return Ok(Value::Str(l_str + &r_str));
                    }
                }

                let (l_val, r_val) = match (left_val, right_val) {
                    (Value::Number(l), Value::Decimal(r)) => (Value::Decimal(l as f64), Value::Decimal(r)),
                    (Value::Decimal(l), Value::Number(r)) => (Value::Decimal(l), Value::Decimal(r as f64)),
                    other => other,
                };

                match (l_val, r_val) {
                    (Value::Number(l), Value::Number(r)) => {
                        match op {
                            // Checked arithmetic: overflow becomes a language-level runtime
                            // error instead of a debug-build panic or a silent release wrap.
                            BinOp::Add => l.checked_add(r).map(Value::Number)
                                .ok_or_else(|| InterpErr::Err(format!("Runtime error: integer overflow in {} + {}", l, r))),
                            BinOp::Subtract => l.checked_sub(r).map(Value::Number)
                                .ok_or_else(|| InterpErr::Err(format!("Runtime error: integer overflow in {} - {}", l, r))),
                            BinOp::Multiply => l.checked_mul(r).map(Value::Number)
                                .ok_or_else(|| InterpErr::Err(format!("Runtime error: integer overflow in {} * {}", l, r))),
                            BinOp::Divide => {
                                if r == 0 { return Err(InterpErr::Err("Runtime error: Division by zero!".to_string())); }
                                // checked_div also catches i64::MIN / -1, which overflows.
                                l.checked_div(r).map(Value::Number)
                                    .ok_or_else(|| InterpErr::Err(format!("Runtime error: integer overflow in {} / {}", l, r)))
                            }
                            BinOp::Modulo => {
                                if r == 0 { return Err(InterpErr::Err("Runtime error: Modulo by zero!".to_string())); }
                                l.checked_rem(r).map(Value::Number)
                                    .ok_or_else(|| InterpErr::Err(format!("Runtime error: integer overflow in {} % {}", l, r)))
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
                    (Value::Decimal(l), Value::Decimal(r)) => {
                        match op {
                            BinOp::Add => Ok(Value::Decimal(l + r)),
                            BinOp::Subtract => Ok(Value::Decimal(l - r)),
                            BinOp::Multiply => Ok(Value::Decimal(l * r)),
                            BinOp::Divide => {
                                if r == 0.0 { return Err(InterpErr::Err("Runtime error: Division by zero!".to_string())); }
                                Ok(Value::Decimal(l / r))
                            }
                            BinOp::Modulo => {
                                if r == 0.0 { return Err(InterpErr::Err("Runtime error: Modulo by zero!".to_string())); }
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
                let right_val = Self::eval_expr(env, right)?;
                match op {
                    UnOp::Negate => match right_val {
                        // checked_neg catches negating i64::MIN, which overflows.
                        Value::Number(n) => n.checked_neg().map(Value::Number)
                            .ok_or_else(|| InterpErr::Err(format!("Runtime error: integer overflow in -{}", n))),
                        Value::Decimal(n) => Ok(Value::Decimal(-n)),
                        _ => Err(InterpErr::Err("Unary '-' can only be applied to Number or Decimal".to_string())),
                    },
                    UnOp::Not => Ok(Value::Bool(!Self::is_truthy_static(&right_val))),
                }
            }

            Expr::Array(elements) => {
                let mut vals = Vec::new();
                for e in elements { vals.push(Self::eval_expr(env, e)?); }
                Ok(Value::Array(Rc::new(RefCell::new(vals))))
            }

            Expr::Dict(pairs) => {
                let mut map = HashMap::new();
                for (k_expr, v_expr) in pairs {
                    let k_val = Self::eval_expr(env, k_expr)?;
                    let v_val = Self::eval_expr(env, v_expr)?;
                    if let Value::Str(key) = k_val { map.insert(key, v_val); } 
                    else { return Err(InterpErr::Err("Dictionary keys must evaluate to String".to_string())); }
                }
                Ok(Value::Dict(Rc::new(RefCell::new(map)))) // ZMIANA
            }

            Expr::IndexGet(container_expr, idx_expr) => {
                let container_val = Self::eval_expr(env, container_expr)?;
                let idx_val = Self::eval_expr(env, idx_expr)?;
                match (container_val, idx_val) {
                    (Value::Array(arr), Value::Number(idx)) => {
                        let arr_ref = arr.borrow();
                        if idx < 0 || idx as usize >= arr_ref.len() {
                            return Err(InterpErr::Err(format!("Array index out of bounds: {}", idx)));
                        }
                        Ok(arr_ref[idx as usize].clone())
                    }
                    (Value::Dict(map), Value::Str(key)) => {
                        let map_ref = map.borrow();
                        Ok(map_ref.get(&key).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(InterpErr::Err("Can only index arrays with numbers or dicts with strings".to_string()))
                }
            }

            Expr::Call(callee, args) => {
                let mut arg_vals = Vec::new();
                for arg in args { arg_vals.push(Self::eval_expr(env, arg)?); }

                let callee_val = Self::eval_expr(env, callee)?;
                // Use the shared helper to execute the function
                return Self::execute_function(callee_val, arg_vals);
            }

            Expr::MethodCall(obj_expr, method_name, args) => {
                let mut arg_vals = Vec::new();
                for arg in args { arg_vals.push(Self::eval_expr(env, arg)?); }

                // A method that mutates its receiver in place (e.g. push) is refused when
                // the receiver is a `let` binding, mirroring the indexed-write protection.
                // This guards the direct case `let xs = [1]; xs.push(2)`. A container reached
                // through a separate `var` alias is still shared and remains mutable.
                if Self::is_mutating_method(method_name)
                    && let Expr::Variable(name) = &**obj_expr
                    && Self::get(env, name).is_some_and(|info| info.is_const)
                {
                    return Err(InterpErr::Err(format!("Cannot call '{}' on '{}': it is declared with 'let'", method_name, name)));
                }

                let obj_val = Self::eval_expr(env, obj_expr)?;

                // The rest are non-mutating methods (pure functions).
                // We look them up in the registered extensions!
                // find_extension returns the pointer by value, so no borrow on `env` is held
                // during the call: extensions like map()/filter() run user callbacks that
                // may assign to variables in this very scope, which needs a mutable borrow.
                let ext_fn = Self::find_extension(env, method_name);
                if let Some(ext_fn) = ext_fn {
                    return ext_fn(obj_val, arg_vals);
                }

                match &obj_val {
                    Value::Array(_) => return Err(InterpErr::Err(format!("Method '{}' not supported on Array", method_name))),
                    Value::Str(_) => return Err(InterpErr::Err(format!("Method '{}' not supported on String", method_name))),
                    _ => return Err(InterpErr::Err(format!("Method '{}' not supported on this type", method_name))),
                }
            }

            Expr::ExecuteCatch(run_body, err_var, catch_body) => {
                match Self::eval_block_as_expr(env, run_body) {
                    Ok(val) => return Ok(val),
                    Err(InterpErr::Err(msg)) => {
                        // The error variable is handler-local: it lives in the handler's own
                        // scope, so it neither overwrites an existing binding of the same
                        // name nor stays visible once the handler finishes.
                        let catch_env = Self::with_parent(Rc::clone(env));
                        if let Some(var_name) = err_var {
                            Self::insert(&catch_env, var_name.clone(), VarInfo {
                                value: Value::Str(msg),
                                is_const: true,
                                type_name: Some("String".to_string())
                            });
                        }
                        return Self::eval_block_in_scope(&catch_env, catch_body);
                    }
                    Err(other_err) => return Err(other_err),
                }
            }

            Expr::If(condition, if_body, else_body) => {
                let cond_value = Self::eval_expr(env, condition)?;
                if Self::is_truthy_static(&cond_value) {
                    return Self::eval_block_as_expr(env, if_body);
                } else {
                    return Self::eval_block_as_expr(env, else_body);
                }
            }

            Expr::NullCoalesce(left, right) => {
                let left_val = Self::eval_expr(env, left)?;
                if let Value::Null = left_val { Self::eval_expr(env, right) } else { Ok(left_val) }
            }

            Expr::Match(target_expr, arms) => {
                let target_val = Self::eval_expr(env, target_expr)?;
                for (pattern, body) in arms {
                    let is_match = if let Some(p_expr) = pattern {
                        let p_val = Self::eval_expr(env, p_expr)?;
                        target_val == p_val
                    } else {
                        true
                    };
                    if is_match {
                        return Self::eval_block_as_expr(env, body);
                    }
                }
                return Err(InterpErr::Err("Match expression exhausted with no matching arm".to_string()));
            }
        }
    }

    fn get_default_value(_type_name: &Option<String>) -> InterpResult<Value> {
        match _type_name {
            Some(t) => match t.as_str() {
                "Number" => Ok(Value::Number(0)),
                "Decimal" => Ok(Value::Decimal(0.0)),
                "String" => Ok(Value::Str("".to_string())),
                "Bool" => Ok(Value::Bool(false)),
                "Array" => Ok(Value::Array(Rc::new(RefCell::new(Vec::new())))),
                "Dict" => Ok(Value::Dict(Rc::new(RefCell::new(HashMap::new())))),
                "Null" => Ok(Value::Null),
                _ => Err(InterpErr::Err(format!("Unknown type: {}", t))),
            },
            None => Err(InterpErr::Err("Cannot infer default value without type".to_string())),
        }
    }

    /// Extension methods that mutate their receiver in place, rather than returning a
    /// new value. Kept as an explicit list so `let` bindings can refuse them; every other
    /// registered extension (map, filter, upper, split, …) is pure and always allowed.
    fn is_mutating_method(name: &str) -> bool {
        matches!(name, "push")
    }

    fn value_matches_type(type_name: &str, value: &Value) -> bool {
        match type_name {
            "Number" => matches!(value, Value::Number(_)),
            "Decimal" => matches!(value, Value::Decimal(_)),
            "String" => matches!(value, Value::Str(_)),
            "Bool" => matches!(value, Value::Bool(_)),
            "Array" => matches!(value, Value::Array(_)),
            "Dict" => matches!(value, Value::Dict(_)),
            "Null" => matches!(value, Value::Null),
            _ => true,
        }
    }

    fn value_type_name(value: &Value) -> &'static str {
        match value {
            Value::Number(_) => "Number",
            Value::Decimal(_) => "Decimal",
            Value::Str(_) => "String",
            Value::Bool(_) => "Bool",
            Value::Array(_) => "Array",
            Value::Dict(_) => "Dict",
            Value::Function(_, _, _) | Value::Builtin(_) => "Function",
            Value::Null => "Null",
        }
    }

    pub fn is_truthy_static(val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0,
            Value::Decimal(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(arr) => !arr.borrow().is_empty(),
            Value::Null => false,
            Value::Function(_, _, _) | Value::Builtin(_) => true,
            Value::Dict(map) => !map.borrow().is_empty(),
        }
    }

    /// Runs a block in a fresh child scope, so anything it declares is discarded on exit.
    /// Returns the value of the last expression statement, which is what makes
    /// `if`, `match` and `execute` usable as expressions.
    fn eval_block_as_expr(env: &Rc<RefCell<Environment>>, stmts: &[Stmt]) -> InterpResult<Value> {
        let block_env = Self::with_parent(Rc::clone(env));
        Self::eval_block_in_scope(&block_env, stmts)
    }

    /// Runs a block directly in the given scope, without creating another one.
    /// Used when the caller has already built the scope in order to seed it first,
    /// as `onError` does with its error variable.
    fn eval_block_in_scope(env: &Rc<RefCell<Environment>>, stmts: &[Stmt]) -> InterpResult<Value> {
        let mut last_val = Value::Null;
        for stmt in stmts {
            match stmt {
                Stmt::ExprStmt(expr) => last_val = Self::eval_expr(env, expr)?,
                _ => Self::eval_stmt(env, stmt)?,
            }
        }
        Ok(last_val)
    }
}
