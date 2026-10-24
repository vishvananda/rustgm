#![feature(decl_macro)]
struct S;
macro install($owner:ident, $member:ident) {
    impl $owner { pub fn $member(&self) -> u8 { 7 } }
}
install!(S, value);
install!(S, value);
fn main() {}
