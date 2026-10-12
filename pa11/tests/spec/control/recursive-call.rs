fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn fact(n:u64)->u64{if n==0{1}else{n*fact(n-1)}} fn entry()->u64{fact(5)}
fn main() {
    check(entry() == 120u64);
}
