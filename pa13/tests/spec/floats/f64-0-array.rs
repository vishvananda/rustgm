fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
fn f(x:[f64;2])->u8{match x{[0.0|1.0,n] if n==7.0=>1,_=>2}}fn main(){check(f([-0.0,7.0])==1&&f([1.0,7.0])==1&&f([0.0/0.0,7.0])==2);}