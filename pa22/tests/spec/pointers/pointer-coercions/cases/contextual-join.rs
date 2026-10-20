fn f(ok: bool, a: &[u8; 2], b: &[u8; 3]) -> *const [u8] {
    if ok { a } else { b }
}
fn main() {}
