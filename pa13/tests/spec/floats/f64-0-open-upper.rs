fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
fn f(x:f64)->u8{match x{LO.. =>1,_=>2}}fn main(){check(f(-1.0/0.0)==1&&f(1.0/0.0)==1&&f(0.0/0.0)==2);}