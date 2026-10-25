#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
impl<'a,T>&'a mut T{fn __rustgm_inherent(self)->u32{7}}
fn run()->u32{let mut n=3u8;(&mut n).__rustgm_inherent()}fn failure(x:u32)->u32{1/x}
fn main(){if run()!=7{let _=failure(0);}}
