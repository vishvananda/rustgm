#![no_implicit_prelude] extern crate self as local; pub mod values { pub const X:u8=7; } mod nested { use crate::local; pub fn read()->u8 { local::values::X } }
