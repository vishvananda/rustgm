fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<'a,T>(x:&'a T)->&'a T{x}fn f<'a,'b>(x:&'a &'b u8)->&'a &'b u8{id(x)}fn main(){let n=13;let p=&n;check(**f(&p)==13);}
