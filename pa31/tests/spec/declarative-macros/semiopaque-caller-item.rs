#![feature(decl_macro)]
#![feature(rustc_attrs)]
mod provider { fn helper() -> u32 { 3 } #[rustc_macro_transparency = "semiopaque"] pub macro value() { helper() } }
fn helper() -> u32 { 5 }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { provider::value!() }
