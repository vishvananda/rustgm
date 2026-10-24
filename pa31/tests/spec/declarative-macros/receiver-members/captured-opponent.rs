#![feature(decl_macro)]
struct S;
macro install($owner:ident, $member:ident, $call:ident) {
    impl $owner { pub fn value(&self) -> u8 { 7 } }
    fn $call() -> u8 { $owner.$member() }
}
install!(S, value, answer);
fn main() { let _ = answer(); }
