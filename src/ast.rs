// src/ast.rs

// Binary operators (mathematic i logic)
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterEq,
    LessEq,
}

// Expressions - they do return some value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Decimal(f64),
    Str(String),
    Bool(bool),
    Variable(String),
    Binary(Box<Expr>, BinOp, Box<Expr>), // left, operator, right
    Array(Vec<Expr>),                    // [1, 2, 3]
    IndexGet(Box<Expr>, Box<Expr>),      // arr[0]
    Call(Box<Expr>, Vec<Expr>),          // function call eg. add(1, 2)
    MethodCall(Box<Expr>, String, Vec<Expr>),   // calling functions with '.' like arr.add(elem)
}

// Instructions (Statements) - the do things but not return any value
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl(String, Option<String>, Option<Expr>),  // var name is TYPE = 5
    Let(String, Option<String>, Option<Expr>),      // let name is TYPE = 5
    Assign(String, Expr),                       // name = expression
    IndexAssign(Box<Expr>, Box<Expr>, Expr),    // arr[0] = 5
    ExprStmt(Expr),                             // expression statement eg. print(x)
    If(Expr, Vec<Stmt>, Vec<Stmt>),             // if (condition) { block } else { block }
    Loop(String, Expr, Expr, Vec<Stmt>),        // loop i from start..end { block }
    FuncDecl(String, Vec<String>, Vec<Stmt>),   // func name(parameters) { body }
    Return(Option<Expr>),                       // return expresiion
    Break,                                      // loop break
    Continue,                                   // loop continue
    Use(String),                                // use io
}
