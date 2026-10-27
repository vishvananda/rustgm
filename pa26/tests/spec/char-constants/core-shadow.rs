fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod core{pub mod primitive{#[allow(non_camel_case_types)]pub struct char;impl char{pub const MIN:u8=7;pub const MAX:u8=9;}}}fn main(){check(core::primitive::char::MIN==7);check(core::primitive::char::MAX==9);check(std::primitive::char::MAX=='\u{10ffff}');}
