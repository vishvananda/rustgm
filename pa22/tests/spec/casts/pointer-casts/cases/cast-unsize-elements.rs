fn f(p: *const [u16; 2]) -> *const [u8] {
    p as *const [u8]
}
fn main() {}
