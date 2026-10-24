#![feature(decl_macro)]
struct S;
macro install($owner:ident, $call:ident) {
    impl $owner { pub fn value(&self) -> u8 { 7 } }
    fn $call() -> u8 {
        let receiver = $owner;
        if receiver.value() == $owner::value(&receiver) { 7 } else { 0 }
    }
}
install!(S, answer);
fn main() { println!("{}", answer()); }
