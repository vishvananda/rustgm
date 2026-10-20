fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn main() {
    let a = [(); 3];
    let s: &[()] = &a;
    let p: *const [()] = s;
    match s {
        [_, rest @ ..] => {
            let q: *const [()] = rest;
            check(p != q);
            check(p > q);
            check(p >= q);
            check(q < p);
            check(q <= p);
        }
        _ => check(false),
    }
}
