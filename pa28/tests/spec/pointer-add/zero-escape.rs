fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}const P:*const u32={let p={let x=7u32;&raw const x};unsafe{p.add(0)}};fn main(){let _=P;}
