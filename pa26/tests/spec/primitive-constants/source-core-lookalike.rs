fn check(b:bool){if !b{let z=0;let _=1/z;}}mod core{pub mod primitive{#[allow(non_camel_case_types)]pub struct u32;impl u32{pub const MAX:u8=9;}}}fn main(){check(core::primitive::u32::MAX==9);}
