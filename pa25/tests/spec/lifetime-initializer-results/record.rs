fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(PartialEq,Eq)]struct V{n:u8,ok:bool}struct S<'a>(&'a u8);impl<'a>S<'a>{const C:V={let n=7;V{n,ok:true}};}fn f<'a>(_:&'a u8,x:V)->bool{match x{S::<'a>::C=>true,_=>false}}fn main(){let n=7;check(f(&n,V{n:7,ok:true}));check(!f(&n,V{n:8,ok:true}));}
