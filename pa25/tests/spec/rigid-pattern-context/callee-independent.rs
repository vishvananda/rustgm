fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const fn value<T>()->u8{7}struct S<T>(T);impl<T>S<T>{const C:u8=value::<T>();}fn run<T>(n:u8)->bool{match n{S::<T>::C=>true,_=>false}}fn main(){check(run::<u8>(7));}
