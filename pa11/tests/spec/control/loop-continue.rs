fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let mut x=0;loop{x+=1;if x<4{continue;}break x;}}
fn main() {
    check(entry() == 4u64);
}
