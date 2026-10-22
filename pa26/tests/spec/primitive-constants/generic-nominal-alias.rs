fn check(b:bool){if !b{let z=0;let _=1/z;}}struct S<T>(T);impl<T>S<T>{const MAX:u8=7;}type W<T>=S<T>;fn main(){check(W::<u16>::MAX==7);}
