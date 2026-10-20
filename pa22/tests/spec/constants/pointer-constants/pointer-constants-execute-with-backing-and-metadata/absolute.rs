fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
const P: *const u8 = 42usize as *const u8;
const Z: *const u8 = 0usize as *const u8;
fn main() {
    check(P as usize == 42);
    check(Z as usize == 0);
}
