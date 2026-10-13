#[derive(PartialEq)]struct X(u8);#[repr(packed)]#[derive(PartialEq)]struct S(X);
fn main() {}
