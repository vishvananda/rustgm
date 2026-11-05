#![feature(no_core,const_trait_impl,const_destruct)]#![no_core]
extern crate base;
pub const fn forward<T:[const]base::Destruct,U:[const]base::Destruct>(value:base::Pair<T,U>)->(T,U){base::parts(value)}
#[unsafe(no_mangle)]pub extern "C" fn run()->u8{let pair=base::parts(base::Pair{first:23u8,second:57u8});pair.1}
