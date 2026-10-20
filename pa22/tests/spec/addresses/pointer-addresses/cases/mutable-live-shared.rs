fn f() {
    let mut x = 7;
    let r = &x;
    let p = &raw mut x;
    let _y = *r;
    let _q = p;
}
fn main() {}
