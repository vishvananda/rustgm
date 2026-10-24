#![feature(decl_macro)]
struct S;
macro install($owner:ident) {
    impl $owner { pub fn value(&self) -> u8 { 7 } }
}
install!(S);
fn main() { let _ = S.value(); }
