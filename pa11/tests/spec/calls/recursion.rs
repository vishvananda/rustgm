fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn fact(x:u128)->u128{if x==0{1}else{x*fact(x-1)}}
fn main() {
    check(fact(20)==2432902008176640000);
}
