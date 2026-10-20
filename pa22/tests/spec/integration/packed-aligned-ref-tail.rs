#[repr(align(64))]
struct A(u8);
#[repr(packed)]
struct S {
    tail: [A],
}
fn f(p: &S) -> &[A] {
    &p.tail
}
fn main() {}
