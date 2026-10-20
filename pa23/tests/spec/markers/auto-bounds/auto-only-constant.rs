fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const P:&dyn Send=&7u8;fn need<T:?Sized+Send>(_:&T)->u8{9}fn main(){check(need(P)==9);}
