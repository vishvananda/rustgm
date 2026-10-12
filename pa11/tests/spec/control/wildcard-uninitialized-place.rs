fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let x:u64;let _=(x);let y:();let _=y;0}
fn main() {
    check(entry() == 0u64);
}
