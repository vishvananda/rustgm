#![feature(decl_macro)]
mod provider {
    pub struct Record { hidden: u32 }
    pub fn make() -> Record { Record { hidden: 23 } }
    pub macro read($record:expr) { $record.hidden }
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    provider::read!(provider::make())
}
