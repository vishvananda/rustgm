#[repr(align(64))]
struct A(u8);
#[repr(packed)]
struct S {
    head: u8,
    tail: [A],
}
fn f(p: &S) -> u8 {
    p.head
}
fn main() {
    let _p: fn(&S) -> u8 = f;
}
