mod num { mod error { pub struct Token(pub u8); } pub use error::Token; } fn main() { let _ = num::Token(7); }
