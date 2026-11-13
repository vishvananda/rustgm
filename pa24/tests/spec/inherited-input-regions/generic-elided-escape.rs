fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<T>(x:&T)->&T{let c=|s:&T|s;c(x)}fn main(){}
