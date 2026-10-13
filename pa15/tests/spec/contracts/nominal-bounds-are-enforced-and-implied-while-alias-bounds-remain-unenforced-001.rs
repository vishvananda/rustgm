struct S<'a:'b,'b>(&'a u8,&'b u8);fn f<'a,'b>(s:S<'a,'b>,x:&'a u8)->&'b u8{x}
fn main() {}
