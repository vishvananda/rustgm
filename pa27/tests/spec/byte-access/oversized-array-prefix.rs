const N:u32={let x=7u32;unsafe{(*(&raw const x as *const [u32;2]))[0]}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==7);}
