fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
const S: *const [u8; 3] = b"abc";
const A: *const [u8; 2] = S as *const [u8; 2];
const P: *const [u8] = A;
const Q: *const [u8] = S;
fn main() {
    check(P != Q);
    check(P < Q);
    check(P as *const u8 == Q as *const u8);
}
