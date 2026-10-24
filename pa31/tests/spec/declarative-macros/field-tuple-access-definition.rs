#![feature(decl_macro)]
mod provider {
    pub struct Record(u32);
    pub fn make() -> Record { Record(43) }
    pub macro read($record:expr) { $record.0 }
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    provider::read!(provider::make())
}
