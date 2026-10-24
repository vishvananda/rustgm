#![feature(decl_macro)]
macro declare($record:ident, $make:ident) {
    pub struct $record { pub value: u32 }
    pub fn $make() -> $record { $record { value: 37 } }
}
declare!(Record, make_record);
macro read($record:expr) { $record.value }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { read!(make_record()) }
