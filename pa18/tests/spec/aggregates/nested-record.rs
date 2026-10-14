fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>{v:T}struct W<T>{s:S<T>}fn main(){let w=W{s:S{v:700u16}};check(w.s.v==700);}
