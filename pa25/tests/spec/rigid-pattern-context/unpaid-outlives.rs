fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a,T>(&'a u8,T);impl<'a,T:'a>S<'a,T>{const C:u8={let n=7;n};}fn run<'a,T>(_:&'a u8,n:u8)->bool{match n{S::<'a,T>::C=>true,_=>false}}fn main(){let n=7;check(run::<&u8>(&n,7));}
