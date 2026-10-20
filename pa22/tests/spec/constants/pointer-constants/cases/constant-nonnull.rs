const P: *const u8 = 42usize as *const u8;
fn f() -> *const u8 {
    P
}
fn main() {}
