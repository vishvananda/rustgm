pub mod values { pub const X:u8=7; } mod declaration { extern crate self as local; pub fn read()->u8 { local::values::X } } mod unrelated { pub fn read()->u8 { local::values::X } }
