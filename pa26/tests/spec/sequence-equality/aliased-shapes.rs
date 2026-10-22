fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A=[u8;2];type S=[u8];fn cmp(a:&A,b:&S)->bool{a.eq(b)}fn main(){check(cmp(&[1,2],&[1,2]));}
