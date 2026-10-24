#![feature(decl_macro)]
mod provider {
    pub struct Record { hidden: u32, pub visible: u32 }
    pub fn make() -> Record { Record { hidden: 1, visible: 2 } }
    pub fn sum(record: Record) -> u32 { record.hidden + record.visible }
    pub macro update($base:expr) { Record { hidden: 6, visible: 23, ..$base } }
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    let base = provider::make();
    provider::sum(provider::update!(base))
}
