#![feature(const_precise_live_drops)]
#![feature(no_core,const_trait_impl,const_destruct)]#![no_core]
extern crate base;
pub const fn forward<T:const base::Destruct>(value:base::Hold<T>)->T{base::part(value)}
#[unsafe(no_mangle)]pub extern "C" fn run()->u8{base::part(base::Hold{value:57u8})}
