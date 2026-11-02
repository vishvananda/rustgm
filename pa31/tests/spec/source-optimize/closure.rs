#![feature(optimize_attribute)]
#![feature(stmt_expr_attributes)]
pub fn f()->u32{let c=#[optimize(size)]||7;c()}
