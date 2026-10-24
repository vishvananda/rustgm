#![feature(decl_macro)]
macro declare($record:ident, $answer:ident) {
    pub struct $record { pub value: u32 }
    #[unsafe(no_mangle)] pub extern "C" fn $answer() -> u32 {
        let record = $record { value: 7 };
        let $record { value: selected } = record;
        let again = $record { value: selected };
        again.value
    }
}
declare!(Record, answer);
