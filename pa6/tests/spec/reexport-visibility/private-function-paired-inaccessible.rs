mod num { mod error { pub struct Token {} #[allow(non_snake_case)] fn Token() -> u8 { 7 } } pub use error::Token; } fn main() { let _ = num::Token(); }
