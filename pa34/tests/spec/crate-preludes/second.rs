#![feature(prelude_import,decl_macro)]
#![no_std]
#![no_main]
pub mod own {
    pub use core::{Clone,Copy,Sized};
    pub struct SecondOnly;
    pub struct Same;
    pub trait Read { fn read(&self)->u8; }
    impl Read for Same { fn read(&self)->u8 { 5 } }
}
#[prelude_import]
use crate::own::*;
pub fn own()->crate::own::SecondOnly { SecondOnly }
pub fn method()->u8 { Same.read() }
