fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn identity<T>(x:T)->T{x}fn main(){check(identity(S)(3).0==3);}
