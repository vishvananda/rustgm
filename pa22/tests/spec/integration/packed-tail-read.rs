#[repr(C, packed)]
struct S {
    tag: u16,
    tail: [u8],
}
fn read(s: &S) -> u8 {
    s.tail[0]
}
fn main() {}
