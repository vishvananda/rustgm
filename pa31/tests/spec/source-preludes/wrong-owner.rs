#![feature(prelude_import)]
mod provider {
    pub enum Choice { Chosen(u32) }
    pub trait Read { fn read(self) -> u32; }
    impl Read for Choice { fn read(self) -> u32 { match self { Choice::Chosen(value) => value } } }
}
mod prelude { pub use crate::provider::{Choice::Chosen, Read}; }
#[prelude_import] fn marker() {}
mod consumer { pub fn run() -> u32 { Chosen(7).read() } }
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u32 { consumer::run() }
