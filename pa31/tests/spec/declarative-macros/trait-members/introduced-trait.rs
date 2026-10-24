#![feature(decl_macro)]
struct S;
macro install($owner:ident, $call:ident) {
    trait Hidden { fn value(&self) -> u8; }
    impl Hidden for $owner { fn value(&self) -> u8 { 7 } }
    fn $call() -> u8 { $owner.value() }
}
install!(S, answer);
fn main() { println!("{}", answer()); }
