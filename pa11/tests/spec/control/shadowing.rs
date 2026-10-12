fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let x=1;{let x=2;let _=x;}x}
fn main() {
    check(entry() == 1u64);
}
