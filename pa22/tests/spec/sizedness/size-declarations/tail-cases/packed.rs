#[repr(packed)]
struct S {
    tag: u16,
    tail: [u8],
}
#[repr(packed)]
struct U<T: Copy> {
    tag: u16,
    tail: [T],
}
fn main() {}
