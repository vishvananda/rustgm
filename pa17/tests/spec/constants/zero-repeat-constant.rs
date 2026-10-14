fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct S;impl Drop for S{fn drop(&mut self){check(false);}}const C:S=S;fn main(){let _a=[C;0];check(true);}
