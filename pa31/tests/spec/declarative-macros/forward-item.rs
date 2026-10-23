#![feature(decl_macro)]
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { forward!() }
pub macro forward() { 11u32 }
