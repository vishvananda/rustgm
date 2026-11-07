#![feature(stmt_expr_attributes)]
pub fn leak()->&'static u8{let x=7u8;let f=#[cold]||&x;f()}
