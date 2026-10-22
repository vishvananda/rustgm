static X:u32=7;const R:&u8=unsafe{&*(&raw const X as *const u8)};const N:u8=*R;fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==7);}
