#![feature(decl_macro)]
mod provider {
    pub struct Record { hidden: u32, pub visible: u32 }
    pub fn make() -> Record { Record { hidden: 1, visible: 2 } }
    pub fn sum(record: Record) -> u32 { record.hidden + record.visible }
    pub macro update($field:ident) { Record { $field: 6, visible: 23, ..make() } }
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    provider::sum(provider::update!(hidden))
}
