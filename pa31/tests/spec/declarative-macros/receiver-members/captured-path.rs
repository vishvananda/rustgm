#![feature(decl_macro)]
struct S;
macro install($owner:ident, $member:ident, $call:ident) {
    impl $owner { pub fn $member(&self) -> u8 { 7 } }
    fn $call() -> u8 { $owner::$member(&$owner) }
}
install!(S, value, answer);
fn main() { println!("{}", answer()); }
