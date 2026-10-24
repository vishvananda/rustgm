#![feature(decl_macro)]
mod provider {
    pub struct Record { hidden: u32, pub visible: u32 }
    pub fn make() -> Record { Record { hidden: 6, visible: 2 } }
    pub fn sum(record: Record) -> u32 { record.hidden + record.visible }
    pub macro update($record:expr) { Record { visible: 23, ..$record } }
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    provider::sum(provider::update!(provider::make()))
}
