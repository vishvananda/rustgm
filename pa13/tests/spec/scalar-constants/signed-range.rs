fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:i8=-5;const HI:i8=LO+8;fn f(x:i8)->u8{match x{LO..=HI=>7,_=>9}}fn main(){check(f(-5)==7&&f(3)==7&&f(4)==9);}