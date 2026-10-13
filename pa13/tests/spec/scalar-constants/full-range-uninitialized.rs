fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:i8=-128;const HI:i8=127;fn main(){let x:i8;match x{LO..=HI=>()}check(true);}