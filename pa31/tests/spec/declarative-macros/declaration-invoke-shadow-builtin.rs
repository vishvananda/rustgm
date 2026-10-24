#![feature(decl_macro)]
pub macro line(){7u32}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{line!()}
