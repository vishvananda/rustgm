fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let mut x=3;false&&{x=9;true};x}
fn main() {
    check(entry() == 3u64);
}
