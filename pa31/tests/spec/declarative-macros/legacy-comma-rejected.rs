#![feature(decl_macro)]
macro_rules! value { () => {31u32}, }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { value!() }
