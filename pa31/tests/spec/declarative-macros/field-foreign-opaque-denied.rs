#![feature(decl_macro)]
macro declare($record:ident) {
    pub struct $record { pub value: u32 }
    impl $record { pub fn new() -> Self { Self { value: 37 } } }
}
declare!(Record);
macro read($record:expr) { $record.value }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { read!(Record::new()) }
