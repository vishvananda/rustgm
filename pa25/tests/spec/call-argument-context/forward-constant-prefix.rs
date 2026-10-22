fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:u8=accept(id);trait K<R>{}struct S;impl K<fn(u8)->u8> for S{}const fn accept<R:Copy>(_:R)->u8 where S:K<R>{3}fn id(x:u8)->u8{x}fn main(){check(N==3);}
