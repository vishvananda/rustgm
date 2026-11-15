#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const trait Compare{fn compare(&self,other:&Self)->u8;}
const impl Compare for u8{fn compare(&self,_other:&Self)->u8{*self}}
impl Compare for u16{fn compare(&self,_other:&Self)->u8{0}}
pub const trait Factory:[const]Compare{fn make()->impl [const]Fn(&Self,&Self)->u8+[const]Destruct{const |a:&Self,b:&Self|Self::compare(a,b)}}
fn main(){}
