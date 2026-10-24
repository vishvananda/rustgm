#![feature(const_trait_impl)]
#![allow(dead_code)]
pub const trait K{fn required()->u8;}pub const trait Q:Sized{fn provided()->u8 where Self:[const]K{<Self as K>::required()}}impl K for u8{fn required()->u8{29}}impl Q for u8{}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Q>::provided()}
