struct S<'a,'b>(&'a &'b u8);struct W<'a,'b>(S<'b,'a>);fn f<'a,'b>(x:W<'a,'b>,y:&'a u8)->&'b u8{y}
fn main() {}
