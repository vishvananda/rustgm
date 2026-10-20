fn f() {
    let mut x = 7;
    let p = &raw mut x;
    x = 9;
    let _q = p;
    let _v = x;
}
fn main() {}
