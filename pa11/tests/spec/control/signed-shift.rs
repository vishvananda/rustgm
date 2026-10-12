fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let x:i8=-8;((x>>1)==-4) as u64}
fn main() {
    check(entry() == 1u64);
}
