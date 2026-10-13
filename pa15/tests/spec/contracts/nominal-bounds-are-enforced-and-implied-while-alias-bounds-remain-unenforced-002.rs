struct S<'a,'b>(&'a u8,&'b u8) where 'a:'b;fn f<'a,'b>(s:S<'a,'b>,x:&'a u8)->&'b u8{x}
fn main() {}
