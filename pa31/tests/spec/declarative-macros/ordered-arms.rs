#![feature(decl_macro)]
pub macro select { (zero) => {0u32}, ($x:expr) => {$x}, }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { select!(zero) + select!(9) }
