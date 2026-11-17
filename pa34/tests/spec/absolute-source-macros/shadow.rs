#![feature(decl_macro)]
#![no_std]
#![no_main]
mod core { pub macro derive($item:item) { compile_error!("wrong local core"); } }
#[::core::prelude::v1::derive()] pub enum Dir {}
pub fn value() -> u8 { 7 }
