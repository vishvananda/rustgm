fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let a=false;let b=true;((a&&(b||a))||(b&&(a||b))) as u64}
fn main() {
    check(entry() == 1u64);
}
