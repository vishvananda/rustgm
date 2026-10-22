fn check(b:bool){if !b{let z=0;let _=1/z;}}#[allow(non_camel_case_types)]struct u32;impl u32{const MAX:u8=7;}fn main(){check(u32::MAX==7);}
