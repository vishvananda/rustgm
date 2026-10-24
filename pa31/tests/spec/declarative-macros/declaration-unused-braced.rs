#![feature(decl_macro)]
pub macro unused{($value:expr)=>{not_resolved($value)},()=>{not_resolved() },}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
