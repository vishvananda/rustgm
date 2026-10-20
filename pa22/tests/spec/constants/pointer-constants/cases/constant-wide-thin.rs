const P: *const u8 = (&[7u8, 9] as *const [u8]) as *const u8;
fn f() -> *const u8 {
    P
}
fn main() {}
