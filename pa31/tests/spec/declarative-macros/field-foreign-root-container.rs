#![feature(decl_macro)]
struct Record { value: u32 }
macro read($record:expr) { $record.value }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { read!(Record { value: 37 }) }
