#![feature(decl_macro)]
fn helper() -> u32 { 37 }
mod provider { pub macro value() { $crate::helper() } }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { provider::value!() }
