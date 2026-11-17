#![no_std]
pub use core::prelude::v1::derive as derive_alias;
pub use core::seven as bang_alias;
#[macro_export]
macro_rules! seven { () => { 7u8 }; }
#[macro_export]
macro_rules! nested { () => { ::core::seven!() }; }
macro_rules! hidden { () => { 8u8 }; }
