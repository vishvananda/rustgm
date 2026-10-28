#![feature(prelude_import)]
mod provider {
    pub enum Choice { Chosen(u32) }
    pub trait Read { fn read(self) -> u32; }
    impl Read for Choice { fn read(self) -> u32 { match self { Choice::Chosen(value) => value } } }
}
mod prelude { pub use crate::provider::{Choice::Chosen, Read}; }
#[prelude_import]
use crate::prelude::*;
mod local { pub fn Chosen(_:u32)->crate::provider::Choice { crate::provider::Choice::Chosen(11) } } mod consumer { use crate::local::*; pub fn run() -> u32 { Chosen(7).read() } }
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u32 { consumer::run() }
