extern crate self as local; pub mod values { pub const X:u8=7; } mod nested { mod local { pub mod values { pub const X:bool=true; } } pub fn read()->bool { local::values::X } }
