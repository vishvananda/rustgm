fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
fn f(x:f64)->f64{match x{n@0.0=>n,_=>1.0}}fn main(){let n=f(-0.0);let p=f(0.0);check(1.0/n<0.0&&1.0/p>0.0);}