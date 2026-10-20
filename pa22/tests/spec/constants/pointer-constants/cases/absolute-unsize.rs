const A: *const [u8; 2] = 42usize as *const [u8; 2];
const P: *const [u8] = A;
fn f() -> *const [u8] {
    P
}
fn main() {}
