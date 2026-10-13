#[derive(PartialEq)]struct A{b:&'static B,bad:X}#[derive(PartialEq)]struct B{a:&'static A}struct X;
fn main() {}
