#[repr(align(64))]
struct A(u8);
#[repr(packed)]
struct S {
    head: A,
    tail: [u8],
}
fn main() {}
