mod num { mod error { pub(crate) struct Token {} #[allow(non_snake_case)] pub fn Token() -> u8 { 7 } } pub use error::Token; } fn inspect(_: &num::Token) {} fn main() { let _ = num::Token(); }
