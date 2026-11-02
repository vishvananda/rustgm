mod num { mod error { pub struct Token(u8); } pub use error::Token; } fn inspect(_: &num::Token) {} fn main() {}
