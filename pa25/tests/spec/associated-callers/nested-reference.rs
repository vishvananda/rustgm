fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a,T>(&'a u8,T);impl<'a,T>S<'a,T>{const B:&'a u8={&7};const C:&'a u8={let n=Self::B;n};}fn f<'a,T>(_:&'a u8,_:T)->&'a u8{S::<'a,T>::C}fn main(){let n=9;check(*f(&n,1u8)==7);}
