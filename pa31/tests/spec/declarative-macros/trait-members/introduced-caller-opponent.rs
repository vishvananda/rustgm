#![feature(decl_macro)]
struct S;
macro install($owner:ident) {
    trait Hidden { fn value(&self) -> u8; }
    impl Hidden for $owner { fn value(&self) -> u8 { 7 } }
}
install!(S);
fn main() { println!("{}", S.value()); }
