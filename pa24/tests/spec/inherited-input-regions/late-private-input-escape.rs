fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a,T>(x:&'a T)->&'a T{let c=|pair:(&T,_)|pair.0;c((x,&7u8))}fn main(){}
