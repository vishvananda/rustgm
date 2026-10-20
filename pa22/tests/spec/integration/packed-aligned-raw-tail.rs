#[repr(align(64))]
struct A(u8);
#[repr(packed)]
struct S {
    tail: [A],
}
fn f(p: &S) -> *const [A] {
    &raw const p.tail
}
fn main() {
    let _p: fn(&S) -> *const [A] = f;
}
