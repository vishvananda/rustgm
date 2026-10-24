#![feature(const_trait_impl)]
#![allow(dead_code)]
pub const trait Q:Sized{fn required()->u8;fn provided()->u8{Self::required()}}impl Q for u8{fn required()->u8{29}}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Q>::provided()}
