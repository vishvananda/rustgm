#![feature(decl_macro)]
mod operators {
    pub trait Value { fn value(&self) -> u8; }
    impl Value for u8 { fn value(&self) -> u8 { 7 } }
    pub trait Other { fn value(&self) -> u8; }
    impl Other for u8 { fn value(&self) -> u8 { 9 } }
}
mod definition { use crate::operators::Other; pub macro literal($x:expr) { $x.value() } }
use operators::Value;
fn answer() -> u8 { return definition::literal!(0u8); }
fn main() { println!("{}", answer()); }
