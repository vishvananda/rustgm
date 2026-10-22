fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}static X:u32=7;const P:*const u32=unsafe{(&raw const X).add(0)};const N:u32=unsafe{*P};fn main(){check(N==7);}
