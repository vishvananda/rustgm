#![feature(decl_macro)]
pub macro select($x:expr) { $x }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { select!(7) }
