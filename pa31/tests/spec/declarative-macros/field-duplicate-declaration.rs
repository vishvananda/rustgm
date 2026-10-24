#![feature(decl_macro)]
macro declare($record:ident) { pub struct $record { pub value: u32, pub value: u32 } }
declare!(Record);
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { 0 }
