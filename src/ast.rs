// src/ast.rs

/// Represents all available binary operators in the language.
/// Covers mathematical operations (arithmetic) and logical/comparison operations.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,        // +
    Subtract,   // -
    Multiply,   // *
    Divide,     // /
    Modulo,     // %
    Equals,     // ==
    NotEquals,  // !=
    GreaterThan,// >
    LessThan,   // <
    GreaterEq,  // >=
    LessEq,     // <=
    And,        // and (logical and)
    Or,         // or (logical or)
}

/// Represents unary operators (applied to a single value).
#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Negate, // - (unary minus, e.g., -5)
    Not,    // not (logical negation, e.g., not true)
}

/// Represents an Abstract Syntax Tree (AST) node for an Expression.
/// Expressions are parts of the code that evaluate to a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// An integer literal (e.g., `10`)
    Number(i64),
    
    /// A floating-point literal (e.g., `3.14`)
    Decimal(f64),
    
    /// A string literal (e.g., `"Hello"`)
    Str(String),
    
    /// A boolean literal (`true` or `false`)
    Bool(bool),
    
    /// A variable reference by its identifier name
    Variable(String),
    
    /// A binary operation: `left [operator] right` (e.g., `a + b`)
    Binary(Box<Expr>, BinOp, Box<Expr>),

    /// A unary operation: `operator right` (e.g., `-5` or `not true`)
    Unary(UnOp, Box<Expr>),
    
    /// An array literal: `[1, 2, 3]`
    Array(Vec<Expr>),
    
    /// An array index access: `arr[0]`
    IndexGet(Box<Expr>, Box<Expr>),
    
    /// An if expression: `if condition { if_body } else { else_body }`
    /// Returns the value of the last expression in the executed block.
    If(Box<Expr>, Vec<Stmt>, Vec<Stmt>),

    /// A standard function call: `add(1, 2)`
    Call(Box<Expr>, Vec<Expr>),
    
    /// A method call on an object/variable using the dot notation: `arr.push(elem)`
    MethodCall(Box<Expr>, String, Vec<Expr>),

    /// An error handling expression: `execute { ... } onError(err) { ... }`
    ExecuteCatch(Vec<Stmt>, Option<String>, Vec<Stmt>),
}

/// Represents an Abstract Syntax Tree (AST) node for a Statement (Instruction).
/// Statements perform actions (like assigning variables or controlling flow) 
/// but do not evaluate to a value themselves.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// A mutable variable declaration: `var name is Type = expression`
    /// (Type and expression are optional, but at least one must be present)
    VarDecl(String, Option<String>, Option<Expr>),
    
    /// An immutable constant declaration: `let name is Type = expression`
    /// (Type and expression are optional, but at least one must be present)
    Let(String, Option<String>, Option<Expr>),
    
    /// An assignment to an existing variable: `name = expression`
    Assign(String, Expr),
    
    /// An assignment to a specific array index: `arr[0] = expression`
    IndexAssign(Box<Expr>, Box<Expr>, Expr),
    
    /// An expression used as a standalone statement (e.g., `print(x)`)
    ExprStmt(Expr),
    
    /// A for-loop iterating over a range of numbers: `loop i from start..end { body }`
    Loop(String, Expr, Expr, Vec<Stmt>),
    
    /// A function declaration: `func name(parameters) { body }`
    FuncDecl(String, Vec<String>, Vec<Stmt>),
    
    /// A return statement: `return expression` (expression is optional, defaults to Null)
    Return(Option<Expr>),
    
    /// A loop control statement to break out of a loop early
    Break,
    
    /// A loop control statement to skip to the next iteration of a loop
    Continue,
    
    /// A module import statement: `use io`
    Use(String),
}
