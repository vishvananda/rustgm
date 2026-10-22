fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T:Copy>S<T>{const C:u8={let n=7;n};}fn run<T:Copy>(n:u8)->bool{match n{S::<T>::C=>true,_=>false}}fn main(){check(run::<u8>(7));check(!run::<u16>(8));}
