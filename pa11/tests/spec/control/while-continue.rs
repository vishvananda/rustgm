fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let mut n=0;let mut sum=0;while n<5{n+=1;if n==3{continue;}sum+=n;}sum}
fn main() {
    check(entry() == 12u64);
}
