fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'b,'c,T>(x:&'c &'b T)->&'b u8{let c=|_|{let y=7u8;&y};c(x)}fn main(){}
