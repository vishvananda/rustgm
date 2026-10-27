extern crate self as local; pub mod values { pub const X:u8=7; } mod nested { pub fn read()->u8 { local::values::X } }
