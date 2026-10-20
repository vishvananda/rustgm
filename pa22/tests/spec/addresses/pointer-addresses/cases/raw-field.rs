struct S {
    a: u8,
}
fn f(p: *const S) -> *const u8 {
    &raw const (*p).a
}
fn main() {}
