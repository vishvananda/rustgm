fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a,'b>(&'a u8,&'b u8);impl<'a:'b,'b>S<'a,'b>{const C:&'b u8={let n:&'a u8=&7;n};}fn f<'a,'b>(_:&'a u8,_:&'b u8,x:&'b u8)->bool{match x{S::<'a,'b>::C=>true,_=>false}}fn main(){}
