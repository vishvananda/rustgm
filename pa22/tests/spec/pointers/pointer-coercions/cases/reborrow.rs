fn f() {
    let mut x = 7;
    let r = &mut x;
    let p: *mut u8 = r;
    *r = 9;
    let _q = r;
    let _p = p;
}
fn main() {}
