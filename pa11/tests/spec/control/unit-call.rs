fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn unit(){} fn entry()->u64{unit();42}
fn main() {
    check(entry() == 42u64);
}
