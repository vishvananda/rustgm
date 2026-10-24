#![feature(decl_macro)]
macro declare($record:ident, $field:ident, $answer:ident) {
    pub struct $record { pub $field: u32 }
    #[unsafe(no_mangle)] pub extern "C" fn $answer() -> u32 {
        let $field = 17u32;
        let record = $record { $field };
        let $record { $field } = record;
        $field
    }
}
declare!(Record, value, answer);
