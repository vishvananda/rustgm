fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T:?Sized>(T);fn make<T>(x:T)->W<T>{W(x)}fn main(){let value:&W<[u8]>=&make([7u8,9u8]);check(value.0[1]==9);}