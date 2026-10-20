static X: u8 = 7;
const P: *const u8 = &raw const X;
fn f() -> *const u8 {
    P
}
fn main() {}
