#[repr(C, packed)]
struct S {
    tag: u8,
    tail: [u16],
}
fn pointer(s: &S) -> *const [u16] {
    &raw const s.tail
}
fn main() {}
