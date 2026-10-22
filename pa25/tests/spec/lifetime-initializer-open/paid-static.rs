fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static VALUE:u8=7;struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:&'a u8={let n=&VALUE;n};}fn f<'a:'static>(_:&'a u8,x:&'a u8)->bool{match x{S::<'a>::C=>true,_=>false}}fn main(){check(f(&VALUE,&7));check(!f(&VALUE,&8));}
