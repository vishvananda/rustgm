fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S{n:usize}static V:S=S{n:3};type A<T=S,const N:usize={V.n}>=T;impl Drop for A{fn drop(&mut self){check(false);}}fn consume(_:S){}fn main(){consume(S{n:V.n});}
