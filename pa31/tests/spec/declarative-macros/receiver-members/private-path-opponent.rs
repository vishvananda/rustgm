#![feature(decl_macro)]
mod definition {
    pub struct S;
    impl S { fn secret(&self) -> u8 { 7 } }
    pub macro call($receiver:expr, $member:ident) { S::$member(&$receiver) }
}
fn main() { let _ = definition::call!(definition::S, secret); }
