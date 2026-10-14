fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl Drop for S{fn drop(&mut self){check(self.0==7);}}type A<T>=T;fn consume(_:A<S>){}fn main(){consume(S(7));}
