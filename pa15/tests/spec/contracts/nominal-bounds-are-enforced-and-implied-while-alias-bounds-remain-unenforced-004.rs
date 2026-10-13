struct S<'a:'b,'b>(&'a u8,&'b u8);fn f<'a,'b>(x:&'a u8,y:&'b u8){let s=S(x,y);}
fn main() {}
