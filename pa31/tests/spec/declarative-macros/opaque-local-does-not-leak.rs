#![feature(decl_macro)]
macro create() { let hidden = 7u32; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { create!(); hidden }
