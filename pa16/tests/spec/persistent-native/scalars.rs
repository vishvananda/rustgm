fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:i128=-170141183460469231731687303715884105728;const B:u128=340282366920938463463374607431768211455;static C:(i128,u128)=(A,B);fn main(){check(C.0==A && C.1==B);}
