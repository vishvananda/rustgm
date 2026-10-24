#![feature(decl_macro)]
mod other{}mod owner{pub(in crate::other) macro hidden(){7u32}}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
