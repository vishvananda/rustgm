#![feature(rustc_attrs)]
#![cfg_attr(all(),rustc_coherence_is_core)]
impl u8{fn __rustgm_inherent(self)->u32{7}}
fn failure(x:u32)->u32{1/x}
fn main(){if 7u8.__rustgm_inherent()!=7{let _=failure(0);}}
