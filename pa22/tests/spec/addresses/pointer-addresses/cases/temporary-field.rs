struct S {
    a: u8,
}
fn f() -> *const u8 {
    &raw const (S { a: 7 }).a
}
fn main() {}
