#![feature(decl_macro)]
mod definition {
    pub mod operators {
        pub trait Value { fn value(&self) -> u8; }
        impl Value for u8 { fn value(&self) -> u8 { 7 } }
    }
    use self::operators::Value;
    pub macro literal($x:expr) { $x.value() }
    pub macro captured($x:expr, $member:ident) { $x.$member() }
}
use definition::operators::Value as _;
fn main() { println!("{}", definition::captured!(0u8, value)); }
