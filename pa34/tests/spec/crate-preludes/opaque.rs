#![feature(prelude_import,decl_macro)]
#![no_std]
#![no_main]
pub mod own {
    pub use core::{Clone,Copy,Sized};
    pub struct FirstOnly;
    pub struct Same;
    pub trait Read { fn read(&self)->u8; }
    impl Read for Same { fn read(&self)->u8 { 3 } }
}
#[prelude_import]
use crate::own::*;
pub fn own()->crate::own::FirstOnly { FirstOnly }
pub fn method()->u8 { Same.read() }
core::opaque!(generated,run);
pub fn checked()->core::provider::Same { generated::run() }
