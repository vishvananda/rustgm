fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(PartialEq,Eq)]enum V{A(u8),B}struct S<'a>(&'a u8);impl<'a>S<'a>{const C:V={let n=7;V::A(n)};}fn f<'a>(_:&'a u8,x:V)->bool{match x{S::<'a>::C=>true,_=>false}}fn main(){let n=7;check(f(&n,V::A(7)));check(!f(&n,V::B));}
