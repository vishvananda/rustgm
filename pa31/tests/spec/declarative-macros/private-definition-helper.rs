#![feature(decl_macro)]
mod provider { fn helper() -> u32 { 3 } pub macro value() { helper() } }
fn helper() -> u32 { 5 }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { provider::value!() }
