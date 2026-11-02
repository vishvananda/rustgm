mod num { mod error { pub struct Token(pub(crate) u8); } pub use error::Token as Alias; } fn main() { let _ = num::Alias(7); }
