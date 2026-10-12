fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn a()->u64{7} fn b()->u64{8} fn entry()->u64{let f:fn()->u64=loop{if true{break a;}break b;};f()}
fn main() {
    check(entry() == 7u64);
}
