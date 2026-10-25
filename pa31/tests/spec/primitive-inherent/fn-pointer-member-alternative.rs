#![feature(rustc_attrs)]
impl fn(u8)->u8{#[rustc_allow_incoherent_impl]fn __rustgm_inherent(self)->u32{self(7) as u32}}fn identity(x:u8)->u8{x}fn failure(x:u32)->u32{1/x}fn main(){if (identity as fn(u8)->u8).__rustgm_inherent()!=7{let _=failure(0);}}
