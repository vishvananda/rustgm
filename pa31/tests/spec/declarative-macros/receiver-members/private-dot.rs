#![feature(decl_macro)]
mod definition {
    pub struct S;
    impl S { fn secret(&self) -> u8 { 7 } }
    pub macro call($receiver:expr) { $receiver.secret() }
}
fn main() { println!("{}", definition::call!(definition::S)); }
