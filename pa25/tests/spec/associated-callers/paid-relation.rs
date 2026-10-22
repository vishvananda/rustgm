fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a,'b,T>(&'a u8,&'b u8,T);impl<'a:'b,'b,T>S<'a,'b,T>{const C:u8={let n=7;n};}fn f<'a:'b,'b,T>(_:&'a u8,_:&'b u8,_:T)->u8{S::<'a,'b,T>::C}fn main(){let n=7;let m=8;check(f(&n,&m,1u8)==7);}
