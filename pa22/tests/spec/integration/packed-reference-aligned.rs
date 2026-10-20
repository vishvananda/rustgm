#[repr(C, packed)]
struct S {
    tag: u8,
    tail: [u16],
}
fn reference(s: &S) -> &[u16] {
    &s.tail
}
fn main() {}
