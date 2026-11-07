#![feature(allow_internal_unstable)]
#[allow_internal_unstable(stmt_expr_attributes)]
macro_rules! pass {($e:expr)=>{$e};}
pub fn observed(value:u8)->u8 {let f=pass!(#[cold] |value:u8|value);f(value)}
