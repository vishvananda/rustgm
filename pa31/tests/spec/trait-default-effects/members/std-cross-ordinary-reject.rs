#![feature(const_trait_impl)]
#![allow(dead_code)]
pub const trait K{fn required()->u8;}pub const trait Q:Sized+K{fn provided()->u8{<Self as K>::required()}}
