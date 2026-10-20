const P: usize = 42usize as *const u8 as usize;
fn f() -> usize {
    P
}
fn main() {}
