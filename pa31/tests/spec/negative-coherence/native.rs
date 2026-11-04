#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
unsafe extern "C"{fn observe(value:u32);}
trait A{}impl A for u32{}struct S;impl !A for S{}
#[rustc_strict_coherence]trait B{fn value()->u32;}
impl<T:A>B for T{fn value()->u32{7}}
impl B for S{fn value()->u32{9}}
fn main(){unsafe{observe(<u32 as B>::value());observe(<S as B>::value());}}
