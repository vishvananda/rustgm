fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{fn make(x:T)->Self{S(x)}fn again(x:T)->Self{<Self>::make(x)}}fn main(){check(S::again(17u8).0==17);}