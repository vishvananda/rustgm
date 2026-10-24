#![feature(decl_macro)]
struct S;
macro install($owner:ident, $call:ident, $value:literal) {
    impl $owner { pub fn value(&self) -> u8 { $value } }
    fn $call() -> u8 { $owner.value() }
}
install!(S, first, 3);
install!(S, second, 4);
fn main() { println!("{}", first() + second()); }
