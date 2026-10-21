fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn take<'a,T:Copy>(x:&'a T)->T{let f=|y:&u8|{let _=y;*x};let q=3;f(&q)}fn main(){let a=7u8;check(take(&a)==7);}
