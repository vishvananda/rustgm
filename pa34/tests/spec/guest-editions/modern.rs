#![no_std]
#![no_main]
unsafe extern "C" {
    pub fn modern_symbol() -> u8;
}
legacy::lexical!();
pub fn from_prelude() -> From2024 {
    From2024
}
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    r#gen()
}
