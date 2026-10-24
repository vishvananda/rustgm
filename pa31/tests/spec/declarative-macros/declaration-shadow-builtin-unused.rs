#![feature(decl_macro)]
pub macro line(){not_a_builtin_body()}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
