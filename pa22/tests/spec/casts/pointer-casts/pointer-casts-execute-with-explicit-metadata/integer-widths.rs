fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn main() {
    let x = -1i8;
    let p = x as *const u8;
    check(p as usize == 18446744073709551615usize);
    let n = (1u128 << 80) | 17;
    let q = n as *const u8;
    check(q as usize == 17);
    check(q as u8 == 17);
}
