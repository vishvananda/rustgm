fn f(p: *mut [u8]) -> *const [u8] {
    &raw const (*p)
}
fn main() {}
