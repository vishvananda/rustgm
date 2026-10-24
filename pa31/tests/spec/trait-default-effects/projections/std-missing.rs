#![feature(const_trait_impl,const_destruct)]
use core::marker::Destruct;
struct D<'a>(&'a mut u8);
impl Drop for D<'_>{fn drop(&mut self){*self.0=7;}}
pub const trait Family<'a>{type Item;fn consume(value:Self::Item){}}
impl<'a> Family<'a> for u8{type Item=D<'a>;}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{
 let mut count=0u8;
 {let d=D(&mut count); <u8 as Family<'_>>::consume(d);}
 count
}
