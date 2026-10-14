fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T>=fn(T)->T;fn increment(x:u16)->u16{x+1}fn call(f:A<u16>,x:u16)->u16{f(x)}fn main(){check(call(increment,345)==346);}
