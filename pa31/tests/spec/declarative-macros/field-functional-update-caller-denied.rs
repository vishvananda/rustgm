#![feature(decl_macro)]
mod provider {
    pub struct Record { hidden: u32, pub visible: u32 }
    pub fn make() -> Record { Record { hidden: 6, visible: 2 } }
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    let record = provider::Record { visible: 23, ..provider::make() };
    record.visible
}
