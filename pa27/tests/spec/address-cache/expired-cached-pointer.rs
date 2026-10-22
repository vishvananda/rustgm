struct S{v:u32}const N:u32=unsafe{let p={let s=S{v:37};&raw const s.v};*p};
fn main(){check(N==37)}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
