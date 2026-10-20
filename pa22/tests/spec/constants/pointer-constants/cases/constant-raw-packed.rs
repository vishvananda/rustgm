#[repr(packed)]
struct S {
    a: u8,
    b: u16,
}
static X: S = S { a: 7, b: 9 };
const P: *const u16 = &raw const X.b;
fn f() -> *const u16 {
    P
}
fn main() {}
