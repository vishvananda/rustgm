#![feature(decl_macro)]
macro declare($record:ident, $field:ident, $answer:ident) {
    pub struct $record { pub $field: u32, pub value: u32 }
    #[unsafe(no_mangle)] pub extern "C" fn $answer() -> u32 {
        let record = $record { $field: 11, value: 20 };
        let $record { $field: captured, value: literal } = record;
        captured + literal
    }
}
declare!(Record, value, answer);
