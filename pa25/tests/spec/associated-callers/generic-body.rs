fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a,T>(&'a u8,T);impl<'a,T>S<'a,T>{const C:u8={let n=7;n};}fn f<'a,T>(_:&'a u8,_:T)->u8{S::<'a,T>::C}fn main(){let n=7;check(f(&n,1u8)==7);check(f(&n,2u16)==7);}
