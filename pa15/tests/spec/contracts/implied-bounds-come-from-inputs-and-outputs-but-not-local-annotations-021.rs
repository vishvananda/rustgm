struct S<'a,'b>(&'a &'b u8);struct W<'a,'b>(S<'b,'a>);fn f<'a,'b>(x:W<'a,'b>,y:&'b u8)->&'a u8{y}
fn main() {}
