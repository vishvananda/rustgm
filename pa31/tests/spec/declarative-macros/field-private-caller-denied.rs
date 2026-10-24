#![feature(decl_macro)]
mod provider {
    pub struct Record { hidden: u32 }
    pub fn make() -> Record { Record { hidden: 23 } }
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { provider::make().hidden }
