#![feature(decl_macro)]
mod provider { macro value() { 23u32 } }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { provider::value!() }
