fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn pass(p: *const str) -> *const str {
    p
}
fn main() {
    let s = "é";
    let p: *const str = s;
    let q: *const str = s;
    check(pass(p) == q);
}
