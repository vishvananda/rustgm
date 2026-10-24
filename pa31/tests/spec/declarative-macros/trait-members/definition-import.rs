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
fn main() { println!("{}", definition::literal!(0u8)); }
