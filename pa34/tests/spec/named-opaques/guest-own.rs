#![feature(no_core,type_alias_impl_trait)]
#![no_core]
extern crate core;
use core::{Value,define_opaque};
pub type Local=impl Value;
#[define_opaque(Local)]pub fn make()->Local { 7u8 }
