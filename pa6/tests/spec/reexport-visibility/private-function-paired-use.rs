mod num { mod error { pub struct Token {} #[allow(non_snake_case)] pub(crate) fn Token() -> u8 { 7 } } pub use error::Token; } fn main() { let _ = num::Token(); }
