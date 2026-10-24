#![feature(decl_macro)]
fn value() -> u32 { 1 }
macro_rules! bridge { () => { value() } }
pub macro outer() {{ pub fn value() -> u32 { 7 } bridge!() }}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { outer!() }
