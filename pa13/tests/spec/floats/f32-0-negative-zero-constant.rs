fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f32=-1.0/0.0;const HI:f32=1.0/0.0;const NAN:f32=0.0/0.0;
const Z:f32=-0.0;fn f(x:f32)->u8{match x{Z=>1,_=>2}}fn main(){check(f(-0.0)==1&&f(0.0)==1&&f(0.0/0.0)==2);}