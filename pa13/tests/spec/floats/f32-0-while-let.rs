fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f32=-1.0/0.0;const HI:f32=1.0/0.0;const NAN:f32=0.0/0.0;
fn f(mut x:f32)->u8{let mut c=0;while let ..0.0=x{c+=1;x=0.0;}c}fn main(){check(f(-1.0)==1&&f(-0.0)==0&&f(0.0/0.0)==0);}