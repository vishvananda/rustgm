#![feature(decl_macro)]
struct S;
macro install($owner:ident, $member:ident) {
    trait Hidden { fn $member(&self) -> u8; }
    impl Hidden for $owner { fn $member(&self) -> u8 { 7 } }
}
install!(S, value);
fn main() { println!("{}", S.value()); }
