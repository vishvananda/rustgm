fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{'a:{if false{break 'a 7;}8}}
fn main() {
    check(entry() == 8u64);
}
