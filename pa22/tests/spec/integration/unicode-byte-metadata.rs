fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
const S: *const str = "é";
const A: *const [u8; 2] = S as *const [u8; 2];
const P: *const [u8] = A;
const Q: *const [u8] = S as *const [u8];
fn main() {
    check(P == Q);
}
