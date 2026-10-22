fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static VALUE:u8=7;struct S<'a>(&'a u8);impl<'a>S<'a>{const C:&'static u8={let n=&VALUE;n};}fn f<'a>(_:&'a u8,x:&'static u8)->bool{match x{S::<'a>::C=>true,_=>false}}fn main(){let n=7;check(f(&n,&7));check(!f(&n,&8));}
