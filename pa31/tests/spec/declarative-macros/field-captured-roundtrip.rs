#![feature(decl_macro)]
macro declare($record:ident, $field:ident, $answer:ident) {
    pub struct $record { pub $field: u32 }
    #[unsafe(no_mangle)] pub extern "C" fn $answer() -> u32 {
        let record = $record { $field: 11 };
        let $record { $field: selected } = record;
        let again = $record { $field: selected };
        again.$field
    }
}
declare!(Record, value, answer);
