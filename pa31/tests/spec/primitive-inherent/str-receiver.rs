#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
impl str{fn __rustgm_inherent(&self)->u32{7}}fn failure(x:u32)->u32{1/x}
fn main(){if "seven".__rustgm_inherent()!=7{let _=failure(0);}}
