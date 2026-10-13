fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f32=-1.0/0.0;const HI:f32=1.0/0.0;const NAN:f32=0.0/0.0;
const A:f32=(1u8 as f32)/2.0;const B:f32=A+0.5;fn f(x:f32)->u8{match x{A..B=>1,_=>2}}fn main(){check(f(0.5)==1&&f(0.75)==1&&f(1.0)==2);}