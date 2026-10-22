const N:usize={let p={let x=7u32;&raw const x as *const [u8;7]};unsafe{match *p{[_,_,_,_,_,_,_]=>7}}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==7);}
