fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{'a:loop{loop{break 'a 9;}}}
fn main() {
    check(entry() == 9u64);
}
