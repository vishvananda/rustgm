fn f(ok: bool, a: *const u8, b: *mut u8) {
    let _p = if ok { a } else { b };
}
fn main() {}
