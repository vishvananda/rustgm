#![feature(const_trait_impl,const_destruct)]
#![allow(incomplete_features,dead_code)]
use std::marker::Destruct;
pub const trait Consume:Sized{fn consume(self)  {}}
struct D<'a>(&'a mut u8);
impl Drop for D<'_>{fn drop(&mut self){*self.0=7;}}
impl Consume for D<'_>{}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{let mut count=0;{let d=D(&mut count);<D<'_>as Consume>::consume(d);}count}
