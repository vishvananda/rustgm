fn f(p: *const u8) -> *mut u8 {
    &raw mut *p
}
fn main() {}
