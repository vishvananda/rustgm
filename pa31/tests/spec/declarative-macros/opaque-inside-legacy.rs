#![feature(decl_macro)]
pub fn value() -> u32 { 3 }
pub macro pick() { value() }
macro_rules! bridge { () => {{ fn value() -> u32 { 9 } pick!() }} }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { bridge!() }
