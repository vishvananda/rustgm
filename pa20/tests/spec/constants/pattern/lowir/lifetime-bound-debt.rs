fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:u8=7;}fn f<'a>(_:&'a u8,x:u8)->bool{match x{<S<'a>>::C=>true,_=>false}}fn main(){}