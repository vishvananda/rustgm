fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let (x)=4;let _=9;x}
fn main() {
    check(entry() == 4u64);
}
