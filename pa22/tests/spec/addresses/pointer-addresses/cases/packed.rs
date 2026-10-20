#[repr(packed)]
struct P {
    a: u8,
    b: u16,
}
fn f(x: &P) -> *const u16 {
    &raw const x.b
}
fn main() {}
