#![feature(decl_macro)]
use provider::value as selected;
mod provider { pub macro value() { 13u32 } }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { selected!() }
