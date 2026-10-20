struct D;
impl Drop for D {
    fn drop(&mut self) {}
}
#[repr(packed)]
struct S {
    tail: [D],
}
fn main() {}
