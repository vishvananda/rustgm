#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const trait Compare{fn compare(&self,other:&Self)->u8;}
const impl Compare for u8{fn compare(&self,_other:&Self)->u8{*self}}
impl Compare for u16{fn compare(&self,_other:&Self)->u8{0}}
pub const fn make<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
const VALUE:u8={let f=make::<u8>();f(&41,&0)};
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{let f=make::<u8>();f(&value,&0)}
fn main(){}
