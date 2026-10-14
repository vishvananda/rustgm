fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a,'b,T:'b>(&'a &'b T);fn wrap<'a,'b>(x:&'a &'b u8)->S<'a,'b,u8>{S(x)}fn main(){let n=19;let p=&n;check(**wrap(&p).0==19);}
