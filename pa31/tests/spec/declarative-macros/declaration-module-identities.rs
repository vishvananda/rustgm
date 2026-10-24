#![feature(decl_macro)]
mod left{pub macro same(){unknown_left()}}mod right{pub macro same(){unknown_right()}}use left::same as a;use right::same as b;
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
