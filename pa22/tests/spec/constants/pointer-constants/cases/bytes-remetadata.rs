const S: *const [u8; 3] = b"abc";
const A: *const [u8; 2] = S as *const [u8; 2];
const P: *const [u8] = A;
fn f() -> *const [u8] {
    P
}
fn main() {}
