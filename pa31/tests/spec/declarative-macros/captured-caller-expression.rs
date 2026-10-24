#![feature(decl_macro)]
mod provider { pub macro value($x:expr) { $x } }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { let local = 17; provider::value!(local) }
