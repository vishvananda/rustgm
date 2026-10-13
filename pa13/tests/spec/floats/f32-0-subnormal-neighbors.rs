fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f32=-1.0/0.0;const HI:f32=1.0/0.0;const NAN:f32=0.0/0.0;
const T:f32=1.401298464324817e-45;const N:f32=-T;fn f(x:f32)->u8{match x{N..0.0=>1,0.0..T=>2,T=>3,_=>4}}fn main(){check(f(-1.401298464324817e-45)==1&&f(-0.0)==2&&f(0.0)==2&&f(1.401298464324817e-45)==3&&f(2.0*1.401298464324817e-45)==4);}