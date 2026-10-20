fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
const W: *const [u16] = &[7u16, 9];
const P: *const [u8] = W as *const [u8];
fn main() {
    check(P == W as *const [u8]);
}
