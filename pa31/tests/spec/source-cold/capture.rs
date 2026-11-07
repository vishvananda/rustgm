#![feature(stmt_expr_attributes)]
pub fn observed(value:u8,flag:bool,fallback:u8)->u8{let check_mask=#[cold]|v:u8|->u8{if flag{v}else{fallback}};check_mask(value)}
