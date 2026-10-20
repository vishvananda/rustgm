fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
const A: *const [u8; 0] = 0usize as *const [u8; 0];
const P: *const [u8] = A;
const B: *const [u8; 1] = 0usize as *const [u8; 1];
const Q: *const [u8] = B;
const Z: *const u8 = &raw const *(0usize as *const u8);
fn main() {
    check(P != Q);
    check(P as *const u8 == Z);
    check(Q as *const u8 == Z);
}
