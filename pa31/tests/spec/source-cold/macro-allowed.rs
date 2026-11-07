#![feature(allow_internal_unstable)]
#[allow_internal_unstable(stmt_expr_attributes)]
macro_rules! create {()=>{#[cold] |value:u8|value};}
pub fn observed(value:u8)->u8 {let f=create!();f(value)}
