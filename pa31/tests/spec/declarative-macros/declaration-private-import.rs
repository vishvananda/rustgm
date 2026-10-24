#![feature(decl_macro)]
mod owner{macro hidden(){7u32}}mod sibling{use crate::owner::hidden;}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
