struct S<'a,T>(&'a u8,T);impl<'a,T>S<'a,T>{const C:&'a u8={let n=7;&n};}fn f<'a,T>(_:&'a u8,_:T)->&'a u8{S::<'a,T>::C}fn main(){let n=9;let _=f(&n,1u8);}
