#![feature(decl_macro)]
pub macro same(){7u32}pub macro same(){8u32}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
