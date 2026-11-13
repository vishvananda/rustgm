fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a,'b,T>(x:&'a T,y:&'b T)->&'a T{let c=|_,z|z;c(x,y)}fn main(){}
