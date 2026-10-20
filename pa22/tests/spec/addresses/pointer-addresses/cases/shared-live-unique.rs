fn f() {
    let mut x = 7;
    let r = &mut x;
    let p = &raw const x;
    *r = 9;
    let _q = p;
}
fn main() {}
