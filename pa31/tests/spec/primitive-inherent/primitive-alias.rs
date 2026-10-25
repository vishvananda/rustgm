#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
type Byte=u8;impl Byte{fn __rustgm_inherent(self)->u32{7}}fn failure(x:u32)->u32{1/x}
fn main(){if 7u8.__rustgm_inherent()!=7{let _=failure(0);}}
