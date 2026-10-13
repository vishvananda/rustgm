fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
const T:f64=4.9406564584124654e-324;const N:f64=-T;fn f(x:f64)->u8{match x{N..0.0=>1,0.0..T=>2,T=>3,_=>4}}fn main(){check(f(-4.9406564584124654e-324)==1&&f(-0.0)==2&&f(0.0)==2&&f(4.9406564584124654e-324)==3&&f(2.0*4.9406564584124654e-324)==4);}