fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{if true{return 12;}99}
fn main() {
    check(entry() == 12u64);
}
