#[derive(PartialEq)]struct X(u8);#[repr(packed(8))]#[derive(PartialEq)]struct S(X);
fn main() {}
