const A: *const [u8; 0] = 0usize as *const [u8; 0];
const P: *const [u8] = A;
fn f() -> *const [u8] {
    P
}
fn main() {}
