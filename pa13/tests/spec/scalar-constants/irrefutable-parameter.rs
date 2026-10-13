fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:u8=0;const HI:u8=255;fn f(LO..=HI:u8)->u8{7}fn main(){check(f(3)==7);}