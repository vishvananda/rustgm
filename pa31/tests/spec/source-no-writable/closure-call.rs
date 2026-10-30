#![feature(rustc_attrs,stmt_expr_attributes)]
pub fn f(x:u32)->u32{let c=#[rustc_no_writable]|v:u32|v;c(x)}
