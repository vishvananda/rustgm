mod num { mod error { pub struct Token(pub(super) u8); } pub use error::Token; pub fn make() { let _ = Token(7); } } fn main() { num::make(); }
