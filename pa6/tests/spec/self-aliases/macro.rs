extern crate self as renamed; pub mod values { pub const X:u8=7; } mod nested { macro_rules! get { () => { renamed::values::X } } pub fn read()->u8 { get!() } }
