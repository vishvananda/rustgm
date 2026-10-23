#![feature(decl_macro)]
macro value() { local }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { let local = 7u32; value!() }
