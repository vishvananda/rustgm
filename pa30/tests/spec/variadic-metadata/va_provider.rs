#[repr(C)] pub struct Pair{pub a:u64,pub b:f64}
unsafe extern "C"{
 pub fn sum(n:u32,...)->i64;
 pub fn sum_float(n:u32,...)->f64;
 pub safe fn ignores(n:u32,...)->u32;
 pub fn record_result(p:Pair,...)->Pair;
 pub fn record_tail(n:u32,...)->u32;
}
pub fn ordinary(n:u32)->u32{n+1}
