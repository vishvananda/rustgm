#![feature(stmt_expr_attributes)]
pub fn read(p:*const u8)->u8{let f=#[cold]|q:*const u8|*q;f(p)}
