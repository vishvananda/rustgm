extern crate self as local; mod values { const X:u8=7; } mod nested { pub fn read()->u8 { local::values::X } }
