#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
impl u8{const __RUSTGM:u32=7;}impl u8{const __RUSTGM:u32=7;}fn failure(x:u32)->u32{1/x}fn main(){if u8::__RUSTGM!=7{let _=failure(0);}}
