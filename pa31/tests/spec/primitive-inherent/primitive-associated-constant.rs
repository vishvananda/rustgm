#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
impl u8{const __RUSTGM_INHERENT:u32=7;}fn failure(x:u32)->u32{1/x}
fn main(){if u8::__RUSTGM_INHERENT!=7{let _=failure(0);}}
