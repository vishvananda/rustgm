#![feature(no_core,type_alias_impl_trait)]
#![no_core]
extern crate core;
use core::{Value,define_opaque};
#[define_opaque(core::Public)]pub fn wrong()->core::Public { 7u8 }
