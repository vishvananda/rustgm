fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let mut x=0;while x<5{x+=1;}x}
fn main() {
    check(entry() == 5u64);
}
