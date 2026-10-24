#![feature(decl_macro)]
mod operators {
    pub trait Value { fn value(&self) -> u8; }
    impl Value for u8 { fn value(&self) -> u8 { 7 } }
    pub trait Other { fn value(&self) -> u8; }
    impl Other for u8 { fn value(&self) -> u8 { 9 } }
}
mod definition { pub macro captured($x:expr,$member:ident) { $x.$member() } }
use operators::Value;
fn answer() -> u8 { const UNUSED: u8 = 0; definition::captured!(0u8,value) }
fn main() { println!("{}", answer()); }
