#![feature(stmt_expr_attributes)]
pub fn observed(value:u8)->u8{let f=#[cold]|v:u8|v;f(value)}
