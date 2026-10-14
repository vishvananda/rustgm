fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct S(u8);impl Drop for S{fn drop(&mut self){check(false);}}const C:S=S(7);fn consume(_:S){}fn main(){consume(C);}
