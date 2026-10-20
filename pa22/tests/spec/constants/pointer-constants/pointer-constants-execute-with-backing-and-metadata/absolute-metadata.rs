fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
const A: *const [u8; 2] = 42usize as *const [u8; 2];
const P: *const [u8] = A;
const B: *const [u8; 3] = 42usize as *const [u8; 3];
const Q: *const [u8] = B;
fn main() {
    check(P != Q);
    check(P < Q);
    check(P as *const u8 as usize == 42);
    check(Q as *const u8 as usize == 42);
}
