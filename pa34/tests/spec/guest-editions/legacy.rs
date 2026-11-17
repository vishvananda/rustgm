#![no_std]
#![no_main]
extern "C" {
    pub fn root_legacy_symbol() -> u8;
}
#[path = "shared.rs"]
pub mod shared;
core::include!("included.rs");
pub fn from_prelude() -> From2021 {
    From2021
}
#[macro_export]
macro_rules! lexical {
    () => {
        pub fn gen() -> u8 {
            21
        }
    };
}
