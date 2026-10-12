fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{if true{if false{1}else{2}}else{3}}
fn main() {
    check(entry() == 2u64);
}
